use core::{future::Future, time::Duration};

use rdxota_protocol::{
    otav1::index::{command, response},
    otav2,
};

use crate::{ControlMessage, RdxOtaClient, RdxOtaClientError, RdxOtaClientIO};

pub trait V1Uploader {
    fn upload(&mut self) -> impl Future<Output = Result<(), RdxOtaClientError>> + Send;
    fn send_command(
        &mut self,
        index: u8,
    ) -> impl Future<Output = Result<(), RdxOtaClientError>> + Send;
    fn recv_status(
        &mut self,
        timeout: Duration,
    ) -> impl Future<Output = Result<u8, RdxOtaClientError>> + Send;
}

impl<'a, 'b, IO: RdxOtaClientIO> V1Uploader for RdxOtaClient<'a, 'b, IO> {
    async fn upload(&mut self) -> Result<(), RdxOtaClientError> {
        let mut last_time = self.io.now_secs();
        let mut cur_time = self.io.now_secs();
        let start_time = self.io.now_secs();

        log::info!(target: "redux-canlink", "Cancel last OTA operation.");
        self.send_command(command::CANCEL).await?;
        if self.recv_status(Duration::from_millis(100)).await? != response::CONTINUE {
            return Err(RdxOtaClientError::V1Error);
        }
        log::info!(target: "redux-canlink", "Start new OTA operation.");
        self.send_command(command::START).await?;
        if self.recv_status(Duration::from_millis(100)).await? != response::CONTINUE {
            return Err(RdxOtaClientError::V1Error);
        }

        for (i, chunk) in self.payload.chunks(8).enumerate() {
            let idx = i * 8;
            let mut data = [0u8; 8];
            data[..chunk.len()].copy_from_slice(chunk);
            // Send the payload
            self.io
                .send(
                    self.id_data(),
                    ControlMessage::new(&data),
                    Duration::from_secs(1),
                )
                .await?;
            // Receive a response
            'retry: loop {
                match self.recv_status(Duration::from_millis(100)).await {
                    Ok(status) => match status {
                        response::ERR => {
                            log::error!(target: "redux-canlink", "OTA error received in response to fw xmit :(");
                            return Err(RdxOtaClientError::V1Error);
                        }
                        response::CONTINUE => {
                            break 'retry;
                        }
                        _ => {
                            continue 'retry;
                        }
                    },
                    Err(e) => {
                        if e == RdxOtaClientError::RecvTimeout {
                            let mut tell_attempt_cnt = 0;
                            self.send_command(command::TELL).await?;
                            let tell = 'recv_data: loop {
                                let tell_attempt = self.io.recv(Duration::from_millis(200)).await;
                                match tell_attempt {
                                    Ok(msg) => {
                                        if msg.length < 5 {
                                            continue 'recv_data;
                                        }
                                        if msg.data[0] == response::ERR {
                                            return Err(RdxOtaClientError::V1Error);
                                        }
                                        if msg.data[0] != response::CONTINUE {
                                            continue 'recv_data;
                                        }
                                        break u32::from_le_bytes(
                                            msg.data[1..5].try_into().unwrap(),
                                        );
                                    }
                                    Err(e) => match e {
                                        crate::RdxOtaIOError::RecvTimeout => {
                                            if tell_attempt_cnt >= 25 {
                                                log::error!(target: "redux-canlink", "OTA full timeout after 25 query attempts, aborting.");
                                                return Err(RdxOtaClientError::RecvTimeout);
                                            }
                                            tell_attempt_cnt += 1;
                                            // try the tell again
                                            self.send_command(command::TELL).await?;
                                            continue 'recv_data;
                                        }
                                        other => {
                                            return Err(other.into());
                                        }
                                    },
                                }
                            };
                            if tell as usize == idx {
                                // device did not receive payload, just repeat the message and retry the receive
                                self.io
                                    .send(
                                        self.id_data(),
                                        ControlMessage::new(&data),
                                        Duration::from_secs(1),
                                    )
                                    .await?;
                                continue 'retry;
                            } else {
                                // go to next packet
                                break 'retry;
                            }
                        } else {
                            return Err(e);
                        }
                    }
                }
            }
            cur_time = self.io.now_secs();
            let pct_progress = idx as f32 * 100_f32 / self.payload.len() as f32;
            if idx % 512 == 0 {
                let speed = (8.0_f32 * 512_f32) / (cur_time - last_time);
                last_time = cur_time;
                self.io.update_progress(i, pct_progress, speed).await;

                log::info!(target: "redux-canlink", "Uploaded {}/{} bytes ({:.2}%) ({:.2} bytes/s)", idx + 8, self.payload.len(), pct_progress, speed);
            }
        }
        log::info!(target: "redux-canlink", "FW successfully transmitted (total time: {} s). Telling device it's done.", cur_time - start_time);
        self.send_command(command::NEXT).await?;
        if self.recv_status(Duration::from_millis(5000)).await? != response::CONTINUE {
            return Err(RdxOtaClientError::V1Error);
        }

        log::info!(target: "redux-canlink", "Telling device to commit OTA update.");
        self.send_command(command::NEXT).await?;
        if self.recv_status(Duration::from_millis(5000)).await? != response::CONTINUE {
            return Err(RdxOtaClientError::V1Error);
        }
        log::info!(target: "redux-canlink", "Commit successful, telling device to reboot.");
        self.send_command(command::NEXT).await?;

        let mut status: Result<u8, RdxOtaClientError> = Err(RdxOtaClientError::RecvTimeout);
        for _ in 0..10 {
            match self.recv_status(Duration::from_secs(1)).await {
                Ok(r) => {
                    if r != response::CONTINUE {
                        status = Ok(r);
                        break;
                    }
                }
                Err(e) => {
                    status = Err(e);
                }
            }
        }
        self.io.reset();

        match status? {
            response::COMPLETE | otav2::index::ctrl::ACK => {
                log::info!(target: "redux-canlink", "Rebooted firmware reports update success.");
                Ok(())
            }
            e => {
                log::error!(target: "redux-canlink", "Rebooted firmware reports error state {}.", e);
                Err(RdxOtaClientError::V1Error)
            }
        }
    }

    async fn send_command(&mut self, index: u8) -> Result<(), RdxOtaClientError> {
        Ok(self
            .io
            .send(
                self.id_to_device(),
                ControlMessage::new(&[index]),
                Duration::from_secs(1),
            )
            .await?)
    }

    async fn recv_status(&mut self, timeout: Duration) -> Result<u8, RdxOtaClientError> {
        let msg = self.io.recv(timeout).await?;
        Ok(msg.data[0])
    }
}
