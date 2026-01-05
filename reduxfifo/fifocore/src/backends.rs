#[cfg(feature = "wpihal-rio")]
pub mod halcan;

#[cfg(target_os = "linux")]
pub mod socketcan;

pub mod rdxusb;
pub mod slcan;
pub mod usb;
pub mod websocket;
pub mod websocket_legacy;

use std::sync::Arc;

use rustc_hash::FxHashMap;
use tokio::sync::watch;

use crate::{
    ReadBuffer, ReduxFIFOMessage, ReduxFIFOSession, ReduxFIFOSessionConfig, WriteBuffer,
    error::Error, logger::LoggerTx,
};

pub trait MessageBackend: Send + core::fmt::Debug {
    /// Open a new [`ReduxFIFOSession`] with this backend.
    fn open_session(
        &mut self,
        msg_count: u32,
        config: ReduxFIFOSessionConfig,
    ) -> Result<ReduxFIFOSession, Error>;
    /// Closes a given [`ReduxFIFOSession`] by its session ID.
    /// This also returns the currently held read buffer
    fn close_session(&mut self, ses: ReduxFIFOSession) -> Result<ReadBuffer, Error>;
    /// Executes a read barrier.
    ///
    /// The bumpvec of pointers is handed to the backend. Control of the previously used [`ReduxFIFOBuffer`]s is handed back to the API caller.
    fn read_barrier(&mut self, data: &mut [ReadBuffer]);
    /// Executes a write barrier.
    /// This executes synchronously.
    ///
    /// The bumpvec of pointers is immediately returned to the caller control-flow-wise, and the backend does not own the underlying buffers.
    fn write_barrier(&mut self, data: &mut WriteBuffer);
    /// Checks if the bus address parameters match this message backend.
    fn params_match(&self, params: &str) -> bool;
    /// Get an RX size notifier for a session.
    fn rx_notifier(&mut self, ses: ReduxFIFOSession) -> Result<watch::Receiver<u32>, Error>;

    fn write_single(&mut self, msg: &ReduxFIFOMessage) -> Result<(), Error>;

    fn sessions(&self) -> Vec<ReduxFIFOSession>;
    fn bus_id(&self) -> u16;
    fn params<'a>(&'a self) -> &'a str;
    fn id_cache(&self) -> IdCache;
    fn max_packet_size(&self) -> usize;

    fn set_logger(&mut self, logger: LoggerTx);
}

/// this is what `backends/*.rs` actually implements
pub trait Backend: core::fmt::Debug + Send {
    type State: 'static;
    /// Start the session.
    /// Enclosed is also an arc/mutex state map, that the backend is responsible for
    /// inserting a [`SessionState`] into
    fn start_session(
        &mut self,
        msg_count: u32,
        config: &ReduxFIFOSessionConfig,
    ) -> Result<Self::State, Error>;

    /// Write multiple messages onto bus
    ///
    /// If not defined, will fallback to impl with [`Backend::write_single`].
    /// Not all backends may have an efficient write-multiple primitive, after all.
    fn write_messages(&mut self, messages: &mut WriteBuffer) {
        let mut status = Ok(());
        let mut written = 0_usize;
        for msg in messages.messages() {
            match self.write_single(msg) {
                Ok(_) => {
                    written += 1;
                }
                Err(e) => {
                    status = Err(e);
                    break;
                }
            }
        }
        messages.meta.messages_written = written as u32;
        messages.set_status(status);
    }
    /// Write single message onto the bus, return Ok on success
    fn write_single(&mut self, msg: &ReduxFIFOMessage) -> Result<(), Error>;
    /// Checks if the bus address parameters match this message backend.
    fn params_match(&self, params: &str) -> bool;
    /// The maximum packet size for this message backend.
    fn max_packet_size(&self) -> usize;
}

#[derive(Debug, Clone, Default)]
pub struct IdCache(pub FxHashMap<u32, u64>);
impl IdCache {
    pub fn update(&mut self, id: u32, ts: u64) {
        let message_key = id & 0x1fff_003f;
        if let Some(ent) = self.0.get_mut(&message_key) {
            *ent = ts;
        } else {
            self.0.insert(message_key, ts);
        }
    }
}

impl serde::Serialize for IdCache {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut seq = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0.iter() {
            seq.serialize_entry(&format!("{k:08x}"), v)?;
        }
        seq.end()
    }
}

#[derive(Debug)]
pub struct SessionTable<S> {
    pub sessions: FxHashMap<ReduxFIFOSession, SessionState<S>>,
    pub id_cache: IdCache,
    pub bus_id: u16,
    pub logger: LoggerTx,
}
impl<S: 'static> SessionTable<S> {
    pub fn ingest_message(&mut self, msg: ReduxFIFOMessage) {
        self.id_cache.update(msg.message_id, msg.timestamp);
        for ses in self
            .sessions
            .values_mut()
            .filter(|ses| ses.config.message_matches(&msg))
        {
            ses.read_buf.add_message(msg);
            ses.update_rx_notifier();
        }
        if let Some(logger) = &mut self.logger {
            logger.try_send(msg).ok();
        }
    }

    pub fn iter_sessions_halcan_use_only<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut SessionState<S>, &mut IdCache, &LoggerTx),
    {
        for session in self.sessions.values_mut() {
            f(session, &mut self.id_cache, &self.logger)
        }
    }
    pub fn new(bus_id: u16) -> Self {
        Self {
            sessions: Default::default(),
            id_cache: Default::default(),
            bus_id,
            logger: None,
        }
    }
}

pub trait BackendOpen: Backend + Sized {
    fn open(
        bus_id: u16,
        _params: &str,
        runtime: tokio::runtime::Handle,
        ses_table: Arc<parking_lot::Mutex<SessionTable<Self::State>>>,
    ) -> Result<Self, Error>;
}

#[derive(Debug)]
pub struct SessionState<S> {
    pub session: ReduxFIFOSession,
    pub config: ReduxFIFOSessionConfig,
    pub read_buf: ReadBuffer,
    pub rx_notifier: watch::Sender<u32>,
    pub backend_state: S,
}

impl<S> SessionState<S> {
    /// Notifies listeners if the rx threshold is reached
    pub fn update_rx_notifier(&self) {
        self.rx_notifier
            .send_replace(self.read_buf.meta.valid_length);
    }

    pub fn swap_buffers(&mut self, swap_buf: &mut ReadBuffer) {
        core::mem::swap(&mut self.read_buf, swap_buf);
        self.update_rx_notifier();
    }
}

/// Session controller for a Bus.
#[derive(Debug)]
pub(crate) struct BusController<B: Backend + core::fmt::Debug> {
    bus_id: u16,
    next_session_id: u32,
    params: String,
    backend: B,
    ses_table: Arc<parking_lot::Mutex<SessionTable<B::State>>>,
    logger: Option<tokio::sync::mpsc::Sender<ReduxFIFOMessage>>,
}
impl<B: BackendOpen> BusController<B>
where
    <B as Backend>::State: core::fmt::Debug + Send,
{
    #[allow(unused)]
    pub fn new(bus_id: u16, params: &str, runtime: tokio::runtime::Handle) -> Result<Self, Error> {
        let ses_table = Arc::new(parking_lot::Mutex::new(SessionTable::new(bus_id)));
        Ok(Self {
            bus_id,
            next_session_id: 0,
            params: params.to_string(),
            backend: B::open(bus_id, params, runtime, ses_table.clone())?,
            ses_table: ses_table,
            logger: None,
        })
    }
}

impl BusController<crate::backends::rdxusb::RdxUsbBackend> {
    pub fn new(
        bus_id: u16,
        params: &str,
        runtime: tokio::runtime::Handle,
        usb_event_loop: Arc<parking_lot::Mutex<usb::UsbEventLoop>>,
    ) -> Result<Self, Error> {
        let ses_table: Arc<parking_lot::Mutex<SessionTable<usb::UsbSessionState>>> =
            Arc::new(parking_lot::Mutex::new(SessionTable::new(bus_id)));
        Ok(Self {
            bus_id,
            next_session_id: 0,
            params: params.to_string(),
            backend: crate::backends::rdxusb::RdxUsbBackend::open(
                bus_id,
                params,
                runtime,
                ses_table.clone(),
                usb_event_loop,
            )?,
            ses_table: ses_table,
            logger: None,
        })
    }
}

impl<B: Backend> MessageBackend for BusController<B>
where
    <B as Backend>::State: core::fmt::Debug + Send,
{
    /// Open a new [`ReduxFIFOSession`] with this backend.
    fn open_session(
        &mut self,
        msg_count: u32,
        config: ReduxFIFOSessionConfig,
    ) -> Result<ReduxFIFOSession, Error> {
        let session_id = self.next_session_id;
        if session_id == u32::MAX {
            return Err(Error::MaxSessionsOpened);
        }

        let mut ses_table = self.ses_table.lock();
        let session = ReduxFIFOSession::from_parts(session_id, self.bus_id);
        if ses_table.sessions.contains_key(&session) {
            return Err(Error::SessionAlreadyOpened);
        }
        let state = self.backend.start_session(msg_count, &config)?;
        ses_table.sessions.insert(
            session,
            SessionState {
                session,
                config,
                read_buf: ReadBuffer::new(session, msg_count),
                backend_state: state,
                rx_notifier: watch::channel(0).0,
            },
        );

        self.next_session_id += 1;
        Ok(session)
    }
    /// Closes a given [`ReduxFIFOSession`] by its session ID.
    /// This also releases control of the associated memory.
    fn close_session(&mut self, ses: ReduxFIFOSession) -> Result<ReadBuffer, Error> {
        let mut ses_table = self.ses_table.lock();
        Ok(ses_table
            .sessions
            .remove(&ses)
            .ok_or(Error::InvalidSessionID)?
            .read_buf)
    }

    /// Executes a read barrier.
    fn read_barrier(&mut self, data: &mut [ReadBuffer]) {
        let mut ses_table = self.ses_table.lock();
        for entry in data {
            let session = entry.session();
            entry.ready_for_read();
            if let Some(state) = ses_table.sessions.get_mut(&session) {
                state.swap_buffers(entry);
            } else {
                entry.set_status(Err(Error::InvalidSessionID));
            }
        }
    }
    /// Executes a write barrier.
    /// This executes synchronously.
    ///
    /// The backend does not own the underlying buffers.
    fn write_barrier(&mut self, data: &mut WriteBuffer) {
        data.ready_for_write();
        self.backend.write_messages(data);
        if let Some(logger) = &mut self.logger {
            let written = data.messages_written();
            for msg in data.messages().iter().take(written) {
                let mut tx_msg = msg.clone();
                tx_msg.flags |= ReduxFIFOMessage::FLAG_TX;
                logger.try_send(tx_msg).ok();
            }
        }
    }
    /// Checks if the bus address parameters match this message backend.
    fn params_match(&self, params: &str) -> bool {
        self.backend.params_match(params)
    }

    fn params<'a>(&'a self) -> &'a str {
        &self.params
    }

    fn id_cache(&self) -> IdCache {
        let ses_table = self.ses_table.lock();
        ses_table.id_cache.clone()
    }

    /// Get an RX size notifier for a session.
    fn rx_notifier(&mut self, ses: ReduxFIFOSession) -> Result<watch::Receiver<u32>, Error> {
        let ses_table = self.ses_table.lock();
        if let Some(entry) = ses_table.sessions.get(&ses) {
            Ok(entry.rx_notifier.subscribe())
        } else {
            Err(Error::InvalidSessionID)
        }
    }

    fn sessions(&self) -> Vec<ReduxFIFOSession> {
        let ses_table = self.ses_table.lock();
        ses_table.sessions.keys().cloned().collect()
    }

    fn bus_id(&self) -> u16 {
        self.bus_id
    }

    fn write_single(&mut self, msg: &ReduxFIFOMessage) -> Result<(), Error> {
        if let Some(logger) = &mut self.logger {
            let mut tx_msg = msg.clone();
            tx_msg.flags |= ReduxFIFOMessage::FLAG_TX;
            logger.try_send(tx_msg).ok();
        }

        self.backend.write_single(&msg)
    }

    fn max_packet_size(&self) -> usize {
        self.backend.max_packet_size()
    }

    fn set_logger(&mut self, logger: LoggerTx) {
        let mut ses_table = self.ses_table.lock();
        ses_table.logger = logger.clone();
        self.logger = logger;
    }
}
