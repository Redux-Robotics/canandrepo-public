use std::{sync::Arc, time::Duration};

use crate::backends::{Backend, BackendOpen, SessionTable};
use crate::error::Error;
use crate::timebase::monotonic_us;
use crate::{ReduxFIFOMessage, ReduxFIFOSessionConfig, log_debug, log_error, log_trace, timebase};
use parking_lot::Mutex;
use wpihal_rio::can::CANStreamMessage;
use wpihal_rio::error::HALError;

#[derive(Debug)]
pub struct HALFIFOSession {
    /// the HAL_CANStreamSession handle
    stream: wpihal_rio::can::StreamSession,
    /// Inbound fifo data buffer, sized to the read_buf count * sizeof(HAL_CANStreamMessage)
    /// Unfortunately we need to do some buffer copying to translate.
    hal_buf: Vec<CANStreamMessage>,
}

impl HALFIFOSession {
    pub fn new(stream: wpihal_rio::can::StreamSession, size: usize) -> Self {
        HALFIFOSession {
            stream,
            hal_buf: vec![Default::default(); size],
        }
    }
}

/// Backend that implements via the WPILib HAL CAN interface.
/// For all intents and purposes this is the RoboRIO CAN bus
///
/// Desktop sim may change what this actually talks to however
#[derive(Debug)]
pub struct HalCanBackend {
    // This is the data mutex.
    read_task: tokio::task::JoinHandle<()>,
}

async fn halcan_read_loop(bus_id: u16, sessions: Arc<Mutex<SessionTable<HALFIFOSession>>>) {
    // unfortunately we can't use tokio as an RTOS.
    // Be really cool if it was viable as one though lmao
    let mut interval = tokio::time::interval(Duration::from_millis(1));
    let min_time = monotonic_us();
    loop {
        {
            let mut logged_messages = false;
            let mut ses_lock = sessions.lock();

            ses_lock.iter_sessions_halcan_use_only(|ses, id_cache, logger| {
                let (count, maybe_err) = ses
                    .backend_state
                    .stream
                    .read_into(&mut ses.backend_state.hal_buf);
                // HACK: there isn't an elegant way to do this currently.
                // SystemCore solves this problem with the socketcan backend but I don't have time to care.
                let should_log = !logged_messages && ses.config.filter_id == 0x0e_0000;

                // translate the messages and add them to the buffer
                for ent in &ses.backend_state.hal_buf[..count.min(ses.backend_state.hal_buf.len())]
                {
                    let mut data = [0u8; 64];
                    data[..8].copy_from_slice(&ent.data);
                    let message_id = ent.messageID;
                    // retimestamp from monotonic into wpilib time
                    let mono_time = ent.timeStamp as i64 * 1000;
                    if mono_time < min_time {
                        // throw out packets from before program start
                        continue;
                    }
                    let timestamp = timebase::retimestamp_from_monotonic(mono_time);
                    let msg = ReduxFIFOMessage {
                        message_id,
                        bus_id,
                        flags: 0,
                        data_size: ent.dataSize,
                        timestamp,
                        data,
                    };

                    ses.read_buf.add_message(msg);

                    // update the id cache
                    id_cache.update(message_id, timestamp);
                    if should_log {
                        if let Some(logger) = logger.as_ref() {
                            logger.try_send(msg).ok();
                        }
                    }
                }

                if count > 0 {
                    ses.update_rx_notifier();
                }
                if let Some(e) = maybe_err {
                    log_error!("Got HALError: {e}, {}", e.0);
                }
                if should_log {
                    logged_messages = true;
                }
            });

            drop(ses_lock);
        }
        interval.tick().await;
    }
}

impl Backend for HalCanBackend {
    type State = HALFIFOSession;
    fn params_match(&self, params: &str) -> bool {
        params.starts_with("halcan")
    }

    fn start_session(
        &mut self,
        msg_count: u32,
        config: &ReduxFIFOSessionConfig,
    ) -> Result<Self::State, Error> {
        log_trace!("config: {:?}, max_messages: {}", config, msg_count);
        match wpihal_rio::can::StreamSession::open(config.filter_id, config.filter_mask, msg_count)
        {
            Ok(stream_session) => Ok(HALFIFOSession::new(stream_session, msg_count as usize)),
            Err(e) => {
                e.send_error();
                Err(Error::HalCanOpenSessionFail)
            }
        }
    }

    fn write_single(&mut self, msg: &ReduxFIFOMessage) -> Result<(), Error> {
        if msg.data_size as usize > self.max_packet_size() {
            return Err(Error::DataTooLong);
        }

        match wpihal_rio::can::send_message(
            msg.message_id,
            msg.data_slice(),
            wpihal_rio::can::SEND_PERIOD_NO_REPEAT,
        ) {
            Ok(()) => Ok(()),
            Err(HALError(wpihal_rio::can::ERR_CAN_BUFFER_OVERRUN)) => Err(Error::BusBufferFull),
            Err(e) => {
                log_error!("ReduxFIFO hal_can write error: {e}");
                Err(Error::BusWriteFail)
            }
        }
    }

    fn max_packet_size(&self) -> usize {
        8
    }
}

impl Drop for HalCanBackend {
    fn drop(&mut self) {
        self.read_task.abort();
        // sessions will all autodrop with the arc and the end of the read task
        // which will also close HAL_CAN_CloseSession
    }
}

impl BackendOpen for HalCanBackend {
    /// This bus is opened more or less unconditionally.
    /// It should generally be assumed that the bus number is 0.
    fn open(
        bus_number: u16,
        _params: &str,
        runtime: tokio::runtime::Handle,
        ses_table: Arc<Mutex<SessionTable<Self::State>>>,
    ) -> Result<Self, Error> {
        // We unconditionally open bus 0, despite the names.
        // On SystemCore this backend isn't supported in favor of the direct SocketCAN backend.
        log_debug!("open halcan: {bus_number}");

        // Initialize the HAL before doing anything else
        wpihal_rio::initialize_common();

        let read_task = runtime.spawn(halcan_read_loop(bus_number, ses_table));

        Ok(HalCanBackend { read_task })
    }
}
