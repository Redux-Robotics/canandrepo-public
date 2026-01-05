use core::{future::Future, time::Duration};

use rdxota_protocol::otav2::{self, Ack, Command, Nack, Response};

use crate::{ControlMessage, RdxOtaClient, RdxOtaClientError, RdxOtaClientIO};

pub trait V2Uploader {
    fn upload(&mut self) -> impl Future<Output = Result<(), RdxOtaClientError>> + Send;
    fn send_command(
        &mut self,
        cmd: Command,
    ) -> impl Future<Output = Result<(), RdxOtaClientError>> + Send;
    fn recv_response(
        &mut self,
        timeout: Duration,
        nack_err: bool,
    ) -> impl Future<Output = Result<Response, RdxOtaClientError>> + Send;
    fn send_recv_chunk_op(
        &mut self,
        cmd: Command,
        tries: u32,
    ) -> impl Future<Output = Result<Option<Nack>, RdxOtaClientError>> + Send;
}

impl<'a, 'b, IO: RdxOtaClientIO> V2Uploader for RdxOtaClient<'a, 'b, IO> {
    async fn upload(&mut self) -> Result<(), RdxOtaClientError> {
        let mut last_time = self.io.now_secs();
        let mut cur_time = self.io.now_secs();
        let start_time = self.io.now_secs();

        self.send_command(Command::Abort).await?;

        log::info!(target: "redux-canlink", "Cancel last OTA operation.");
        self.recv_response(Duration::from_millis(100), false)
            .await
            .ok(); // we don't care what this is 
        // run stat on inode 0
        self.send_command(Command::Stat(0)).await?;
        log::info!(target: "redux-canlink", "Stat firmware upload slot.");
        let stat = match self
            .recv_response(Duration::from_millis(1000), true)
            .await?
        {
            Response::Stat(stat) => stat,
            other => return Err(RdxOtaClientError::V2UnexpectedResponse(other)),
        };

        if !stat.inode_executable || !stat.inode_exists {
            log::error!(target: "redux-canlink", "Firmware is not executable or does not exist!!!");
            return Err(RdxOtaClientError::V2InvalidSlot(0));
        }

        if !stat.inode_writeable {
            if !stat.requires_dfu {
                log::error!(target: "redux-canlink", "Firmware slot is not writeable!");
                return Err(RdxOtaClientError::V2FirmwareSlotNotWritable);
            } else {
                log::info!(target: "redux-canlink", "Rebooting device to DFU mode\n");
                self.send_command(Command::SysCtl([
                    otav2::index::sysctl::BOOT_TO_DFU,
                    0,
                    0,
                    0,
                    0,
                    0,
                    0,
                ]))
                .await?;
                // wait. this is like, 15-25% of the entire OTA duration. right here. lmao.
                // it can probably be sped up if the message layer is modded to support awaiting until it receives enumerate packets
                self.io.sleep(Duration::from_millis(500)).await?;

                log::info!(target: "redux-canlink", "Check device is in DFU mode\n");
                self.send_command(Command::DeviceState).await?;
                let device_in_dfu = match self
                    .recv_response(Duration::from_millis(1000), true)
                    .await?
                {
                    Response::DeviceState(s) => s[0] & 0b1 == 1,
                    other => {
                        return Err(RdxOtaClientError::V2UnexpectedResponse(other));
                    }
                };
                if !device_in_dfu {
                    log::error!(target: "redux-canlink", "Device did not switch to DFU mode!\n");
                    return Err(RdxOtaClientError::V2CouldNotSwitchToDFU);
                }

                self.send_command(Command::Stat(0)).await?;
                log::info!(target: "redux-canlink", "Ensure firmware is writeable\n");
                let stat = match self
                    .recv_response(Duration::from_millis(1000), true)
                    .await?
                {
                    Response::Stat(stat) => stat,
                    other => return Err(RdxOtaClientError::V2UnexpectedResponse(other)),
                };

                if !stat.inode_writeable {
                    log::error!(target: "redux-canlink", "Firmware slot is not writeable despite being in DFU mode!\n");
                    return Err(RdxOtaClientError::V2FirmwareSlotNotWritable);
                }
            }
        }
        log::info!(target: "redux-canlink", "Start new OTAv2 upload.\n");
        self.send_command(Command::Upload(0)).await?;

        let mut chunk_size = match self
            .recv_response(Duration::from_millis(1000), true)
            .await?
        {
            Response::Ack(ack) => match ack {
                Ack::TransferStart(chunk_size) => chunk_size as usize & (!8),
                other => {
                    return Err(RdxOtaClientError::V2UnexpectedAck(other));
                }
            },
            other => return Err(RdxOtaClientError::V2UnexpectedResponse(other)),
        }; // (8-align packets)

        let max_chunk_size = chunk_size;
        log::info!(target: "redux-canlink", "Using chunksize {}\n", chunk_size);

        let fw_len = self.payload.len();
        let mut i = 0usize;

        let mut failures = 0;
        let mut successes = 0;
        const MIN_CHUNK_SIZE: usize = 8;
        while i < fw_len {
            let mut crc = 0xffffffff;
            let chunk_len = (i + chunk_size).min(fw_len) - i;

            // scratch_buf's len is the max size of the transport packet.
            let max_packet_len = self.scratch_buf.len().min(self.io.transport_size());
            // within-chunk index
            let mut j = 0usize;

            while j < chunk_len {
                let packet_len = (j + max_packet_len).min(chunk_len) - j;
                self.scratch_buf.fill(0);
                self.scratch_buf[..packet_len]
                    .copy_from_slice(&self.payload[i + j..i + j + packet_len]);
                let buf = &self.scratch_buf[..packet_len.max(MIN_CHUNK_SIZE)];
                crc = rdxcrc::crc32_mpeg2_pad(crc, buf);

                // for testing purposes let's have a 1/1024 chance of just not xmitting a packet
                self.io
                    .send_data(self.id_data(), buf, Duration::from_millis(10))
                    .await?;

                j += packet_len;
            }
            self.io.sleep(Duration::from_micros(1000)).await?;
            self.io.reset();

            if let Some(crc_nack) = self
                .send_recv_chunk_op(Command::VerifyChunk(crc), 100)
                .await?
            {
                match crc_nack {
                    Nack::ChunkCRC32Fail => {
                        log::warn!(target: "redux-canlink", "failed to upload fw[{}..{}], retrying...", i, i + chunk_len);
                        failures += 1;
                        successes = 0;
                        if failures >= 2 {
                            if chunk_size > MIN_CHUNK_SIZE {
                                failures = 0;
                                chunk_size >>= 1;
                            } else if failures > 20 {
                                log::error!(target: "redux-canlink", "OTA is unable to make progress, aborting.");
                                return Err(RdxOtaClientError::V2Stalled);
                            }
                        }
                        'crc_fail_filter: loop {
                            match self
                                .send_recv_chunk_op(Command::ClearChunk(crc), 200)
                                .await?
                            {
                                None => {
                                    break 'crc_fail_filter;
                                }
                                Some(n) => {
                                    if n == Nack::ChunkCRC32Fail {
                                        continue 'crc_fail_filter;
                                    } else {
                                        log::error!(target: "redux-canlink", "Can't clear chunk and get a consistent state, aborting.");
                                        return Err(RdxOtaClientError::V2Nack(n));
                                    }
                                }
                            }
                        }
                        // don't increment the chunk ctr, we need to restart
                        continue;
                    }
                    other => {
                        return Err(RdxOtaClientError::V2Nack(other));
                    }
                }
            } else {
                // we need up to 5 seconds to let legacy canandmags take 4 seconds to initialize their internal OTA stack.
                if let Some(n) = self
                    .send_recv_chunk_op(Command::CommitChunk(crc), 500)
                    .await?
                {
                    log::error!(target: "redux-canlink", "Commit failure!\n");
                    return Err(RdxOtaClientError::V2Nack(n));
                }

                // we win!
                successes += 1;
                failures = 0;
                let new_chunk_size = if successes >= 4 && chunk_size <= max_chunk_size {
                    successes = 0;
                    (chunk_size << 1).min(max_chunk_size)
                } else {
                    chunk_size
                };

                cur_time = self.io.now_secs();
                let speed = chunk_len as f32 / (cur_time - last_time);
                let pct_progress = (i + chunk_len) as f32 * 100.0f32 / (fw_len as f32);
                last_time = cur_time;
                let written = i + chunk_len;
                log::info!(target: "redux-canlink", "Uploaded {written}/{fw_len} bytes ({pct_progress:.2}%) ({speed:.2} bytes/s)\n");
                self.io.update_progress(written, pct_progress, speed).await;

                i += chunk_size;
                // we need to delay applying the new chunksize until AFTER we've already moved i by the amount of the previous chunk
                chunk_size = new_chunk_size;
            }
        }

        log::info!(
            target: "redux-canlink",
            "FW successfully transmitted (total time: {} s). Telling device it's done.",
            cur_time - start_time
        );

        // let's collect spurious acks
        self.send_command(Command::Finish).await?;
        match self
            .recv_response(Duration::from_millis(5000), true)
            .await?
        {
            Response::Ack(_) => {}
            other => {
                return Err(RdxOtaClientError::V2UnexpectedResponse(other));
            }
        }
        log::info!(target: "redux-canlink", "Commit successful, telling device to reboot.");
        self.send_command(Command::DeviceState).await?;
        // check if the device is ready to reboot.
        'reboot_ready: loop {
            match self
                .recv_response(Duration::from_millis(1000), true)
                .await?
            {
                Response::DeviceState(state) => {
                    if state[1] != 0 {
                        log::error!(target: "redux-canlink", "Error: device still stuck in upload mode!!!");
                        return Err(RdxOtaClientError::V2UnexpectedResponse(
                            Response::DeviceState(state),
                        ));
                    } else {
                        break 'reboot_ready;
                    }
                }
                Response::Ack(_) => {}
                other => {
                    return Err(RdxOtaClientError::V2UnexpectedResponse(other));
                }
            }
        }

        self.send_command(Command::SysCtl([
            otav2::index::sysctl::BOOT_NORMALLY,
            0,
            0,
            0,
            0,
            0,
            0,
        ]))
        .await?;

        log::info!(target: "redux-canlink", "Firmware uploaded finished. If lights are still blue, try power cycling.\n");
        Ok(())
    }

    async fn send_command(&mut self, cmd: Command) -> Result<(), RdxOtaClientError> {
        self.io
            .send(
                self.id_to_device(),
                ControlMessage {
                    data: cmd.into(),
                    length: 8,
                },
                Duration::from_millis(10),
            )
            .await?;
        Ok(())
    }

    async fn recv_response(
        &mut self,
        timeout: Duration,
        nack_err: bool,
    ) -> Result<Response, RdxOtaClientError> {
        loop {
            let msg = self.io.recv(timeout).await?;
            if msg.length >= 8 {
                let response = Response::from(msg.data);
                break if nack_err {
                    match response {
                        Response::Nack(n) => Err(RdxOtaClientError::V2Nack(n)),
                        Response::Unknown(u) => Err(RdxOtaClientError::V2InvalidResponse(u)),
                        r => Ok(r),
                    }
                } else {
                    Ok(response)
                };
            }
        }
    }

    async fn send_recv_chunk_op(
        &mut self,
        cmd: Command,
        tries: u32,
    ) -> Result<Option<Nack>, RdxOtaClientError> {
        // send & verify chunk command
        let (chunk_op, sent_idx) = ChunkOperation::extract_value(cmd)?;

        'retry: for _ in 0..tries {
            self.send_command(cmd).await?;
            match (
                chunk_op,
                self.recv_response(Duration::from_millis(10), false).await,
            ) {
                (ChunkOperation::ClearChunk, Ok(Response::Ack(Ack::ChunkCleared(v))))
                | (ChunkOperation::VerifyChunk, Ok(Response::Ack(Ack::ChunkVerified(v))))
                | (ChunkOperation::CommitChunk, Ok(Response::Ack(Ack::ChunkCommitted(v)))) => {
                    if v == 0 || v == sent_idx {
                        return Ok(None);
                    }
                }
                (_, Ok(Response::Nack(n))) => {
                    return Ok(Some(n));
                }
                (_, Ok(_)) | (_, Err(RdxOtaClientError::RecvTimeout)) => {
                    continue 'retry;
                }
                (_, Err(e)) => {
                    return Err(e);
                }
            }
        }
        Err(RdxOtaClientError::RecvTimeout)
    }
}

pub fn str_for_nack(nack: &Nack) -> &'static str {
    match nack {
        Nack::InvalidArgument => "Invalid argument for operation",
        Nack::InvalidFileIndex => "Invalid file index",
        Nack::OperationAborted => "Operation aborted",
        Nack::DeviceBusy => "Device busy",
        Nack::AccessDenied => "Access denied",
        Nack::ChunkCRC32Fail => "Chunk CRC mismatch",
        Nack::CommitFail => "Chunk commit failure",
        Nack::BufferOverrun => "Chunk buffer overrun",
        Nack::UnknownOTA => "Unknown payload-layer error",
        Nack::HeaderMagicFail => "Header magic failure; did you upload a valid RdxOTA file?",
        Nack::HeaderVersionFail => "Firmware file uses incorrect RdxOTA version",
        Nack::HeaderProductMismatch => "Firmware file is not for this device",
        Nack::HeaderEciesKeySigFail => "Key signature failure",
        Nack::HeaderHmacFail => "Header HMAC failure",
        Nack::BlockHeaderMagicFail => "Block header magic failure",
        Nack::BlockHeaderHmacFail => "Block header HMAC failure",
        Nack::BlockHeaderInvalid => "Block header invalid",
        Nack::DataAddressInvalid => "Data address invalid",
        Nack::DataInvalid => "Block data invalid",
        Nack::EraseFail => "Page erase failure",
        Nack::FlashFail => "Flash operation failure",
        Nack::FinalVerificationFailure => "Final verification failure",
        Nack::NotDone => "Incomplete data uploaded",
        Nack::Unknown => "Unknown error",
    }
}

pub fn str_for_ack(ack: &Ack) -> &'static str {
    match ack {
        Ack::Ok => "Ok",
        Ack::TransferStart(_) => "Transfer start",
        Ack::ChunkVerified(_) => "Chunk verified",
        Ack::ChunkCommitted(_) => "Chunk committed",
        Ack::ChunkCleared(_) => "Chunk cleared",
        Ack::Unknown => "Unknown",
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChunkOperation {
    VerifyChunk,
    CommitChunk,
    ClearChunk,
}

impl ChunkOperation {
    pub fn extract_value(value: Command) -> Result<(Self, u32), RdxOtaClientError> {
        match value {
            Command::VerifyChunk(n) => Ok((Self::VerifyChunk, n)),
            Command::CommitChunk(n) => Ok((Self::CommitChunk, n)),
            Command::ClearChunk(n) => Ok((Self::ClearChunk, n)),
            _ => Err(RdxOtaClientError::IOError("this is a bug in otav2 impl!")),
        }
    }
}
