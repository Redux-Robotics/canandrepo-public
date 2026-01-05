use std::sync::{Arc, atomic::AtomicU32};

use rustc_hash::FxHashMap;
use tokio::{sync::watch, task::JoinHandle};

use crate::{
    ReadBuffer, ReduxFIFOMessage, ReduxFIFOSession, ReduxFIFOSessionConfig, Session, WriteBuffer,
    backends::{self, MessageBackend},
    error::Error,
};

#[allow(unused)]
#[derive(Debug, Clone)]
struct DropAbortHandle(Arc<JoinHandle<()>>);
impl Drop for DropAbortHandle {
    fn drop(&mut self) {
        self.0.abort();
    }
}

/// The core of the FIFO event loop.
///
/// Be warned that its raw APIs are un-ergonomic for any programming language.
///
/// Including Rust, unfortunately.
#[derive(Debug, Clone)]
pub struct FIFOCore {
    /// we wrap this in a Mutex so that FIFOCore can be [`Sync`]
    buses: Arc<parking_lot::Mutex<FxHashMap<u16, Box<dyn MessageBackend>>>>,
    runtime: tokio::runtime::Handle,
    id: u32,
    usb_evloop: Arc<parking_lot::Mutex<backends::usb::UsbEventLoop>>,
    #[allow(unused)]
    usb_hotplug: DropAbortHandle,
    loggers: Arc<parking_lot::Mutex<FxHashMap<u16, crate::logger::Logger>>>,
}

impl PartialEq for FIFOCore {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl Eq for FIFOCore {}

static FIFOCORE_ID: AtomicU32 = AtomicU32::new(0);

impl FIFOCore {
    pub fn new(runtime: tokio::runtime::Handle) -> Self {
        let (usb_evloop, usb_hotplug) = {
            let usb_evloop = Arc::new(parking_lot::Mutex::new(backends::usb::UsbEventLoop::new()));
            let usb_hotplug = DropAbortHandle(Arc::new(
                runtime.spawn(backends::usb::UsbEventLoop::hotplug(usb_evloop.clone())),
            ));
            (usb_evloop, usb_hotplug)
        };

        let inst = Self {
            buses: Default::default(),
            runtime,
            id: FIFOCORE_ID.fetch_add(1, core::sync::atomic::Ordering::SeqCst),
            usb_evloop,
            usb_hotplug,
            loggers: Default::default(),
        };
        #[cfg(feature = "wpihal-rio")]
        inst.open_or_get_bus("halcan")
            .expect("Could not open wpihalcan");

        #[cfg(feature = "systemcore")]
        for bus in ["can_s0", "can_s1", "can_s2", "can_s3", "can_s4"] {
            inst.open_or_get_bus(&format!("socketcan:{bus}"))
                .expect(&format!("Could not open {bus}"));
        }

        inst
    }

    pub fn runtime(&self) -> tokio::runtime::Handle {
        self.runtime.clone()
    }

    /// Searches for a bus matching the parameters.
    pub fn bus_matching_params(&self, params: &str) -> Option<u16> {
        let buses = self.buses.lock();

        for ent in buses.values() {
            if ent.params_match(params) {
                return Some(ent.bus_id());
            }
        }
        None
    }

    /// Opens a new bus with the given parameters or returns an error..
    pub fn open_or_get_bus(&self, params: &str) -> Result<u16, Error> {
        if let Some(id) = self.bus_matching_params(params) {
            return Ok(id);
        }
        self.open_bus(params)
    }

    /// Underlying open bus machinery.
    fn open_bus(&self, params: &str) -> Result<u16, Error> {
        let mut buses = self.buses.lock();
        if buses.len() >= u16::MAX as usize {
            return Err(Error::MaxBusesOpened);
        }
        let next_id = buses.keys().max().map_or(0, |v| *v + 1); //buses.len() as u16;

        let backend: Result<Box<dyn MessageBackend>, Error> = if params.starts_with("halcan") {
            #[cfg(feature = "wpihal-rio")]
            {
                Ok(Box::new(backends::BusController::<
                    backends::halcan::HalCanBackend,
                >::new(
                    next_id, params, self.runtime.clone()
                )?))
            }
            #[cfg(not(feature = "wpihal-rio"))]
            {
                crate::log_error!(
                    "halcan backend not supported without WPILib support compiled in"
                );
                Err(Error::BusNotSupported)
            }
        } else if params.starts_with("socketcan") {
            #[cfg(target_os = "linux")]
            {
                Ok(Box::new(backends::BusController::<
                    backends::socketcan::SocketCanBackend,
                >::new(
                    next_id, params, self.runtime.clone()
                )?))
            }
            #[cfg(not(target_os = "linux"))]
            {
                crate::log_error!("socketcan backend not supported on non-linux");
                Err(Error::BusNotSupported)
            }
        } else if params.starts_with("rdxusb") {
            Ok(Box::new(backends::BusController::<
                backends::rdxusb::RdxUsbBackend,
            >::new(
                next_id,
                params,
                self.runtime.clone(),
                self.usb_evloop.clone(),
            )?))
        } else if params.starts_with("websocket:") {
            Ok(Box::new(backends::BusController::<
                backends::websocket_legacy::WebSocketBackend,
            >::new(
                next_id, params, self.runtime.clone()
            )?))
        } else if params.starts_with("ws:") {
            Ok(Box::new(backends::BusController::<
                backends::websocket::WebSocketBackend,
            >::new(
                next_id, params, self.runtime.clone()
            )?))
        } else if params.starts_with("slcan:") {
            Ok(Box::new(backends::BusController::<
                backends::slcan::SlcanBackend,
            >::new(
                next_id, params, self.runtime.clone()
            )?))
        } else {
            crate::log_error!("Unknown bus backend {params}");
            Err(Error::InvalidBus)
        };
        buses.insert(next_id, backend?);
        Ok(next_id)
    }

    /// Closes a bus if exists
    /// Accomplished by dropping the SessionController, which will in turn drop the Backend
    pub fn close_bus(&self, bus_id: u16) -> Result<(), Error> {
        let mut buses = self.buses.lock();
        buses.remove(&bus_id).ok_or(Error::BusClosed)?;
        Ok(())
    }

    pub fn buses(&self) -> Vec<u16> {
        let buses = self.buses.lock();
        buses.keys().cloned().collect()
    }

    /// this is an Escape Hatch to let you do things in a locked fifocore context
    pub fn with_buses<'a, T>(
        &'a self,
        mut f: impl FnMut(parking_lot::MutexGuard<'a, FxHashMap<u16, Box<dyn MessageBackend>>>) -> T,
    ) -> T {
        f(self.buses.lock())
    }

    pub fn max_packet_size(&self, bus_id: u16) -> Result<usize, Error> {
        let buses = self.buses.lock();
        buses
            .get(&bus_id)
            .ok_or(Error::InvalidBus)
            .map(|b| b.max_packet_size())
    }

    pub fn sessions(&self, bus_id: u16) -> Vec<ReduxFIFOSession> {
        let buses = self.buses.lock();
        buses
            .get(&bus_id)
            .ok_or(Error::InvalidBus)
            .map_or(Vec::new(), |b| b.sessions())
    }

    /// Opens a new session with the given initial read buffer.
    pub fn open_session(
        &self,
        bus_id: u16,
        msg_count: u32,
        config: ReduxFIFOSessionConfig,
    ) -> Result<ReduxFIFOSession, Error> {
        let mut buses = self.buses.lock();
        let bus = buses.get_mut(&bus_id).ok_or(Error::InvalidBus)?;
        bus.open_session(msg_count, config)
    }

    pub fn open_managed_session(
        &self,
        bus_id: u16,
        msg_count: u32,
        config: ReduxFIFOSessionConfig,
    ) -> Result<Session, Error> {
        unsafe {
            Ok(Session::wrap(
                self.clone(),
                self.open_session(bus_id, msg_count, config)?,
            ))
        }
    }

    pub fn open_managed_session_by_str(
        &self,
        bus_str: &str,
        msg_count: u32,
        config: ReduxFIFOSessionConfig,
    ) -> Result<Session, Error> {
        let bus_id = self.open_or_get_bus(bus_str)?;
        self.open_managed_session(bus_id, msg_count, config)
    }

    /// Closes a session.
    /// If the associated bus is already closed, return an error.
    pub fn close_session(&self, ses: ReduxFIFOSession) -> Result<ReadBuffer, Error> {
        let mut buses = self.buses.lock();
        let bus = buses.get_mut(&ses.bus_id()).ok_or(Error::InvalidBus)?;
        bus.close_session(ses)
    }

    /// Executes a read barrier.
    /// This assumes all [`ReadBuffer`]s are passed in are associated with the same bus.
    pub fn read_barrier(&self, bus_id: u16, data: &mut [ReadBuffer]) -> Result<(), Error> {
        let mut buses = self.buses.lock();
        let bus = buses.get_mut(&bus_id).ok_or(Error::InvalidBus)?;
        bus.read_barrier(data);

        Ok(())
    }

    /// Executes a multi-bus read barrier.
    /// For each slice in the read buffer, the bus ID is determined from the first entry.
    pub fn read_barrier_multibus<'a>(
        &self,
        data: impl Iterator<Item = &'a mut [ReadBuffer]>,
    ) -> Result<(), Error> {
        let mut buses = self.buses.lock();
        for buffer_list in data {
            let Some(buf0) = buffer_list.get(0) else {
                continue;
            };
            let bus_id = buf0.session().bus_id();

            let bus = buses.get_mut(&bus_id).ok_or(Error::InvalidBus)?;
            bus.read_barrier(buffer_list);
        }
        Ok(())
    }

    pub fn write_barrier(&self, data: &mut [WriteBuffer]) {
        let mut buses = self.buses.lock();
        for buffer in data {
            let bus_id = buffer.meta.bus_id as u16;
            buffer.ready_for_write();
            let Some(bus) = buses.get_mut(&bus_id) else {
                buffer.set_status(Err(Error::InvalidBus));
                return;
            };
            bus.write_barrier(buffer);
        }
    }

    pub fn write_single(&self, msg: &ReduxFIFOMessage) -> Result<(), Error> {
        let mut buses = self.buses.lock();
        let bus = buses.get_mut(&msg.bus_id).ok_or(Error::InvalidBus)?;
        bus.write_single(msg)
    }

    /// Returns an RX buffer size listener.
    /// Return a [`watch::Receiver`] to wait on until ready.
    /// If the session is invalid, return [`Error`]
    pub fn rx_notifier(&self, ses: ReduxFIFOSession) -> Result<watch::Receiver<u32>, Error> {
        let mut buses = self.buses.lock();
        let bus = buses.get_mut(&ses.bus_id()).ok_or(Error::InvalidBus)?;
        bus.rx_notifier(ses)
    }

    /// TODO: this is terrible.
    ///
    /// Needs:
    /// * auto-renaming
    /// * ability to hook multiple buses into one logger
    pub fn open_log(&self, log_path: std::path::PathBuf, bus: u16) -> Result<(), Error> {
        let time_sec = crate::timebase::now_us() as f64 / 1_000_000.0_f64;
        let actual_log_path = if log_path.is_dir() {
            if !log_path.exists() {
                crate::log_error!("Log file folder {} doesn't exist!!!", log_path.display());
                return Err(Error::Unknown);
            }
            let dt: chrono::DateTime<chrono::Utc> = std::time::SystemTime::now().into();

            let dt_fmt = dt.format("%Y_%M_%dT%H_%M_%S");
            log_path.join(format!("rdxlog_bus{bus}_{dt_fmt}_{time_sec:.06}.rdxlog"))
        } else {
            log_path
        };
        let mut buses = self.buses.lock();
        let bus_inst = buses.get_mut(&bus).ok_or(Error::InvalidBus)?;
        let logger = crate::logger::Logger::new(actual_log_path, self.runtime().clone());
        bus_inst.set_logger(logger.sender());
        drop(buses);
        let mut loggers = self.loggers.lock();
        loggers.insert(bus, logger);

        Ok(())
    }

    pub fn close_log(&self, bus_id: u16) -> Result<(), Error> {
        let mut loggers = self.loggers.lock();
        loggers.remove(&bus_id);
        drop(loggers);
        let mut buses = self.buses.lock();
        let bus_inst = buses.get_mut(&bus_id).ok_or(Error::InvalidBus)?;
        bus_inst.set_logger(None);

        Ok(())
    }
}
