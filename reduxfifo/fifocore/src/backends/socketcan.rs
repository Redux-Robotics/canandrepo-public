//! Socketcan backend
//!
//! ## Data model
//! This matches on two types of buses: socketcan:{bus-name} and socketcanfd:{bus-name}
//!
//! ## Opening a bus
//! When the bus is opened, a single TX write task for the corresponding bus is created.
//! A single RX read task is also opened.
//!
//! Sessions can be opened from this bus
//!
use std::{
    sync::Arc,
    time::{Duration, SystemTime},
};

use parking_lot::Mutex;
use socketcan::{
    Frame, Socket as _, SocketOptions,
    id::{FdFlags, id_to_canid_t},
};

use crate::{
    MessageIdBuilder, ReduxFIFOMessage, ReduxFIFOSessionConfig, WriteBuffer,
    backends::{Backend, BackendOpen, SessionTable},
    error::Error,
    log_debug, log_error, log_trace, timebase,
};
use embedded_can::Frame as _;

fn socketcan_id(value: &ReduxFIFOMessage) -> socketcan::Id {
    unsafe {
        if value.short_id() {
            socketcan::Id::Standard(socketcan::StandardId::new_unchecked(
                (value.message_id & 0x7ff) as u16,
            ))
        } else {
            socketcan::Id::Extended(socketcan::ExtendedId::new_unchecked(
                value.message_id & 0x1fff_ffff,
            ))
        }
    }
}

impl TryFrom<&ReduxFIFOMessage> for socketcan::CanFrame {
    type Error = Error;
    fn try_from(value: &ReduxFIFOMessage) -> Result<Self, Error> {
        let data = value.data_slice();
        let id = socketcan_id(value);

        Ok(if value.err() {
            Self::Error(
                socketcan::CanErrorFrame::new_error(id_to_canid_t(id), data)
                    .map_err(|_| Error::DataTooLong)?,
            )
        } else if value.rtr() {
            Self::Remote(
                socketcan::CanRemoteFrame::new_remote(id, value.data_size as usize)
                    .ok_or(Error::DataTooLong)?,
            )
        } else {
            Self::Data(socketcan::CanDataFrame::new(id, data).ok_or(Error::DataTooLong)?)
        })
    }
}

impl TryFrom<&ReduxFIFOMessage> for socketcan::CanAnyFrame {
    type Error = Error;
    fn try_from(value: &ReduxFIFOMessage) -> Result<Self, Error> {
        let data = value.data_slice();

        if value.no_fd() {
            return Ok(socketcan::CanFrame::try_from(value)?.into());
        }
        let id = socketcan_id(value);
        Ok(if value.err() {
            Self::Error(
                socketcan::CanErrorFrame::new_error(id_to_canid_t(id), data)
                    .map_err(|_| Error::DataTooLong)?,
            )
        } else if value.rtr() {
            Self::Remote(
                socketcan::CanRemoteFrame::new_remote(id, value.data_size as usize)
                    .ok_or(Error::DataTooLong)?,
            )
        } else {
            let mut flags = FdFlags::empty();
            flags.set(FdFlags::FDF, !value.no_fd());
            flags.set(FdFlags::BRS, !value.no_brs());

            Self::Fd(socketcan::CanFdFrame::with_flags(id, data, flags).ok_or(Error::DataTooLong)?)
        })
    }
}

#[derive(Debug)]
enum CanBus {
    /// you have NO IDEA how badly i wanted to call this `CanTuah`
    Can2(socketcan::tokio::CanSocketTimestamp),
    /// sigh
    CanFd(socketcan::tokio::CanFdSocketTimestamp),
}

impl CanBus {
    pub fn open(bus: &str, fd: bool) -> Result<CanBus, Error> {
        let open_fail = |e| {
            log_trace!("Failed to open socketcan iface `{bus}`: {e}");
            Error::FailedToOpenBus
        };
        let addr = socketcan::CanAddr::from_iface(bus).map_err(|e| {
            log_trace!("Failed to acquire socketcan iface `{bus}`: {e}");
            Error::InvalidBus
        })?;

        if fd {
            let bus = socketcan::tokio::CanFdSocketTimestamp::open_with_timestamping_mode(
                &addr,
                socketcan::socket::TimestampingMode::Hardware,
            )
            .map_err(open_fail)?;
            let _ = bus.set_loopback(false);
            Ok(Self::CanFd(bus))
        } else {
            let bus = socketcan::tokio::CanSocketTimestamp::open_with_timestamping_mode(
                &addr,
                socketcan::socket::TimestampingMode::Hardware,
            )
            .map_err(open_fail)?;
            let _ = bus.set_loopback(false);
            Ok(Self::Can2(bus))
        }
    }

    pub fn is_fd(&self) -> bool {
        match self {
            CanBus::Can2(_) => false,
            CanBus::CanFd(_) => true,
        }
    }

    async fn read_frame(
        &self,
    ) -> Result<(socketcan::frame::CanAnyFrame, Option<SystemTime>), std::io::Error> {
        match self {
            CanBus::Can2(sock) => sock
                .read_frame()
                .await
                .map(|(frame, ts)| (frame.into(), ts)),
            CanBus::CanFd(sock) => sock
                .read_frame()
                .await
                .map(|(frame, ts)| (frame.into(), ts)),
        }
    }

    pub async fn recv_msg(
        &self,
        state: &SocketCanBackendState,
    ) -> Result<ReduxFIFOMessage, std::io::Error> {
        let (frame, ts) = loop {
            break match tokio::time::timeout(Duration::from_millis(500), self.read_frame()).await {
                Ok(Ok(msg)) => msg,
                Ok(Err(e)) => {
                    return Err(e);
                }
                Err(_) => {
                    // We enforce this timeout to check if the device still exists.
                    // This is because we can't naturally figure out if the bus is actually gone.
                    let _ = socketcan::CanAddr::from_iface(&state.bus_str)
                        .map_err(|_| std::io::Error::from(std::io::ErrorKind::NetworkDown))?;
                    continue;
                }
            };
        };

        let mut data = [0u8; 64];
        let frame_data = frame.data();
        data[..frame_data.len()].copy_from_slice(frame_data);
        let timestamp = match ts {
            Some(s) => timebase::retimestamp_from_monotonic(
                s.duration_since(std::time::SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_micros() as i64,
            ),
            None => timebase::now_us() as u64,
        };

        let mut flags = 0;
        if self.is_fd() {
            if matches!(frame, socketcan::CanAnyFrame::Normal(..)) {
                flags |= ReduxFIFOMessage::FLAG_NO_FD;
            }
            if let socketcan::CanAnyFrame::Fd(fd_frame) = frame
                && !fd_frame.is_brs()
            {
                flags |= ReduxFIFOMessage::FLAG_NO_BRS;
            }
        }

        Ok(ReduxFIFOMessage {
            message_id: MessageIdBuilder::new(frame.id_word())
                .err(frame.is_error_frame())
                .rtr(frame.is_remote_frame())
                .short_id(!frame.is_extended())
                .build(),
            bus_id: state.bus_id,
            flags,
            data_size: frame.dlc() as u8,
            timestamp,
            data,
        })
    }

    pub fn write(&self, frame: &ReduxFIFOMessage) -> Result<(), Error> {
        let result = match self {
            Self::Can2(bus) => {
                let tu_frame: socketcan::CanFrame = frame.try_into()?;
                bus.inner().write_frame(&tu_frame)
            }
            Self::CanFd(bus) => {
                let fd_frame: socketcan::CanAnyFrame = frame.try_into()?;
                bus.inner().write_frame(&fd_frame)
            }
        };

        result.map_err(|e| {
            if e.raw_os_error() == Some(105) || e.kind() == std::io::ErrorKind::WouldBlock {
                // 105 => "No buffer space available"
                Error::BusBufferFull
            } else {
                log_error!("Failed to write packet to socketcan: {e}");
                Error::BusWriteFail
            }
        })
    }

    async fn reopen_bus(state: &SocketCanBackendState) -> Self {
        loop {
            log_debug!("Attempting to open SocketCAN bus `{}`", state.bus_str);
            let Ok(new_bus) = Self::open(&state.bus_str, state.fd) else {
                tokio::time::sleep(Duration::from_millis(50)).await;
                continue;
            };
            log_debug!("SocketCAN bus {} opened!", state.bus_str);
            break new_bus;
        }
    }
}

#[derive(Debug, Clone)]
struct SocketCanBackendState {
    bus_str: String,
    bus_id: u16,
    fd: bool,
}

async fn socketcan_read_loop(
    state: SocketCanBackendState,
    write_bus: Arc<Mutex<Option<Arc<CanBus>>>>,
    ses_table: Arc<Mutex<SessionTable<()>>>,
) {
    log_debug!("Opened SocketCAN bus `{}`", state.bus_str);
    let maybe_bus = write_bus.lock().clone();
    let mut bus = match maybe_bus {
        Some(bus) => bus,
        None => {
            let new_bus = Arc::new(CanBus::reopen_bus(&state).await);
            write_bus.lock().replace(new_bus.clone());
            new_bus
        }
    };

    loop {
        let msg = match bus.recv_msg(&state).await {
            Ok(msg) => msg,
            Err(e) => {
                log_error!(
                    "Failed to read msg: {e}; attempting to open SocketCAN bus `{}`",
                    state.bus_str
                );
                write_bus.lock().take();
                bus = Arc::new(CanBus::reopen_bus(&state).await);
                write_bus.lock().replace(bus.clone());
                continue;
            }
        };

        let mut ses_lock = ses_table.lock();
        ses_lock.ingest_message(msg);
        drop(ses_lock);
    }
}

#[derive(Debug)]
pub struct SocketCanBackend {
    /// we need this for the write path
    state: SocketCanBackendState,
    /// read loop task
    read_task: tokio::task::JoinHandle<()>,
    /// bus
    write_bus: Arc<Mutex<Option<Arc<CanBus>>>>,
}

impl BackendOpen for SocketCanBackend {
    fn open(
        bus_number: u16,
        params: &str,
        runtime: tokio::runtime::Handle,
        ses_table: Arc<Mutex<SessionTable<()>>>,
    ) -> Result<Self, Error> {
        log_debug!("open socketcan: {bus_number}");
        let state = match params.split_once(":") {
            Some(("socketcan", bus)) => SocketCanBackendState {
                bus_str: bus.to_string(),
                bus_id: bus_number,
                fd: false,
            },
            Some(("socketcan.fd", bus)) => SocketCanBackendState {
                bus_str: bus.to_string(),
                bus_id: bus_number,
                fd: true,
            },
            Some((invalid_0, invalid_1)) => {
                log_error!("Invalid SocketCAN bus string {invalid_0}:{invalid_1}.");
                log_error!("Expected `socketcan[.fd]:{{bus name here}}");
                return Err(Error::BusNotSupported);
            }
            None => {
                return Err(Error::BusNotSupported);
            }
        };

        let write_bus = if tokio::runtime::Handle::try_current().is_ok() {
            // if we're in a tokio runtime, open it directly to avoid double-block
            CanBus::open(&state.bus_str, state.fd).ok().map(Arc::new)
        } else {
            // if we're not, have the tokio runtime do it
            runtime
                .block_on((async || CanBus::open(&state.bus_str, state.fd))())
                .ok()
                .map(Arc::new)
        };
        let write_bus = Arc::new(Mutex::new(write_bus));

        let read_task = runtime.spawn(socketcan_read_loop(
            state.clone(),
            write_bus.clone(),
            ses_table,
        ));

        Ok(Self {
            state,
            read_task,
            write_bus,
        })
    }
}

impl Backend for SocketCanBackend {
    type State = ();
    fn params_match(&self, params: &str) -> bool {
        match params.split_once(":") {
            Some(("socketcan", bus)) if bus == self.state.bus_str && !self.state.fd => true,
            Some(("socketcan.fd", bus)) if bus == self.state.bus_str && self.state.fd => true,
            _ => false,
        }
    }

    fn start_session(
        &mut self,
        _msg_count: u32,
        _config: &ReduxFIFOSessionConfig,
    ) -> Result<(), crate::error::Error> {
        Ok(())
    }

    fn write_messages(&mut self, messages: &mut WriteBuffer) {
        let mut status = Ok(());
        let mut written = 0_usize;
        let write_bus = self.write_bus.lock(); //.as_ref().ok_or(Error::BusWriteFail)?.write(msg)
        if let Some(bus) = write_bus.as_ref() {
            for msg in messages.messages() {
                match bus.write(msg) {
                    Ok(_) => {
                        written += 1;
                    }
                    Err(e) => {
                        status = Err(e);
                        break;
                    }
                }
            }
        } else {
            status = Err(Error::BusWriteFail);
        };

        messages.meta.messages_written = written as u32;
        messages.set_status(status);
    }

    fn write_single(&mut self, msg: &ReduxFIFOMessage) -> Result<(), Error> {
        self.write_bus
            .lock()
            .as_ref()
            .ok_or(Error::BusWriteFail)?
            .write(msg)
    }

    fn max_packet_size(&self) -> usize {
        if self.state.fd { 64 } else { 8 }
    }
}

impl Drop for SocketCanBackend {
    fn drop(&mut self) {
        self.read_task.abort();
    }
}
