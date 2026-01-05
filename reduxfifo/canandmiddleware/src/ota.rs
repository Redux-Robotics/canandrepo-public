use std::{
    collections::VecDeque,
    sync::Arc,
    time::{Duration, Instant},
};

use axum::{
    extract::{Path, State},
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
};
use rdxota_client::{ControlMessage, RdxOtaClient, RdxOtaClientIO, RdxOtaIOError};
use tokio::{sync::watch, task::JoinHandle};

use crate::{log::*, rest_server::AppState};
use fifocore::{
    FIFOCore, ReadBuffer, ReduxFIFOMessage, ReduxFIFOSessionConfig, Session, error::Error,
};

/// Glue between reduxfifo and rdxota-client
pub struct ClientIO {
    fifocore: FIFOCore,
    session: Session,
    bus: u16,
    polling_interval: Duration,
    status: Arc<watch::Sender<OtaFlashStatus>>,
    msg_buffer: VecDeque<ReduxFIFOMessage>,
    next_buf: ReadBuffer,
    max_packet_size: usize,
    start_ts: Instant,
}

impl ClientIO {
    pub fn open(
        fifocore: FIFOCore,
        bus: u16,
        id: u32,
        status: Arc<watch::Sender<OtaFlashStatus>>,
    ) -> Result<Self, Error> {
        let session = fifocore.open_managed_session(
            bus,
            64,
            ReduxFIFOSessionConfig::new(
                (id & 0x1fff003f) | ((rdxota_protocol::OTA_MESSAGE_TO_HOST as u32) << 6),
                0x1fffffff,
            ),
        )?;
        let next_buf = session.read_buffer(64);
        let max_packet_size = fifocore.max_packet_size(bus)?;

        Ok(Self {
            fifocore,
            session,
            bus,
            polling_interval: Duration::from_micros(1000),
            status,
            msg_buffer: VecDeque::default(),
            next_buf,
            max_packet_size,
            start_ts: Instant::now(),
        })
    }

    async fn send_msg(
        &self,
        msg: &ReduxFIFOMessage,
        timeout: Duration,
    ) -> Result<(), RdxOtaIOError> {
        let start = Instant::now();
        while Instant::now() - start < timeout {
            match self.fifocore.write_single(&msg) {
                Ok(()) => {
                    return Ok(());
                }
                Err(Error::BusBufferFull) => {
                    // tokio more like suckio am i right
                    tokio::task::yield_now().await;
                    std::thread::sleep(self.polling_interval);
                    continue;
                }
                Err(e) => {
                    return Err(RdxOtaIOError::Other(e.message()));
                }
            }
        }
        Err(RdxOtaIOError::SendTimeout)
    }
}

impl RdxOtaClientIO for ClientIO {
    async fn send(
        &mut self,
        id: u32,
        msg: ControlMessage,
        timeout: core::time::Duration,
    ) -> Result<(), RdxOtaIOError> {
        let mut data = [0_u8; 64];
        data[..msg.length as usize].copy_from_slice(&msg.data[..msg.length as usize]);
        let msg = ReduxFIFOMessage::id_data(self.bus, id, data, msg.length, 0);
        self.send_msg(&msg, timeout).await
    }

    async fn send_data(
        &mut self,
        id: u32,
        msg: &[u8],
        timeout: core::time::Duration,
    ) -> Result<(), RdxOtaIOError> {
        if msg.len() > self.transport_size() {
            return Err(RdxOtaIOError::Other(
                "Message length is too large for transport layer size",
            ));
        }
        let mut data = [0_u8; 64];
        data[..msg.len()].copy_from_slice(msg);
        let msg = ReduxFIFOMessage::id_data(self.bus, id, data, msg.len() as u8, 0);

        self.send_msg(&msg, timeout).await
    }

    async fn recv(
        &mut self,
        timeout: core::time::Duration,
    ) -> Result<ControlMessage, RdxOtaIOError> {
        if let Some(msg) = self.msg_buffer.pop_front() {
            return Ok(msg.into());
        }

        let Ok(mut notifier) = self.session.rx_notifier() else {
            return Err(RdxOtaIOError::Cancelled);
        };
        loop {
            match tokio::time::timeout(timeout, notifier.wait_for(|size| *size > 0)).await {
                Ok(Ok(p)) => {
                    drop(p);
                } // holding this stupid ass object WILL deadlock the rest of the system.
                Ok(Err(_)) => {
                    return Err(RdxOtaIOError::Cancelled);
                }
                Err(_) => {
                    return Err(RdxOtaIOError::RecvTimeout);
                }
            };

            self.session
                .read_barrier(&mut self.next_buf)
                .map_err(|e| RdxOtaIOError::Other(e.message()))?;
            for msg in self.next_buf.iter() {
                self.msg_buffer.push_back(*msg);
            }
            if let Some(msg) = self.msg_buffer.pop_front() {
                return Ok(msg.into());
            }
        }
    }

    async fn sleep(&mut self, timeout: core::time::Duration) -> Result<(), RdxOtaIOError> {
        tokio::time::sleep(timeout).await;
        Ok(())
    }

    fn reset(&mut self) {
        self.msg_buffer.clear();
        let Ok(notifier) = self.session.rx_notifier() else {
            return;
        };
        let value = notifier.borrow().clone();
        if value > 0 {
            let _ = self.session.read_barrier(&mut self.next_buf);
        }
    }

    fn now_secs(&self) -> f32 {
        (Instant::now() - self.start_ts).as_secs_f32()
    }

    async fn update_progress(&mut self, written: usize, pct_progress: f32, speed: f32) {
        self.status.send_replace(OtaFlashStatus {
            state: OtaFlashState::Running,
            written,
            pct_progress: pct_progress as f64,
            speed: speed as f64,
            error_text: None,
        });
    }

    fn transport_size(&self) -> usize {
        self.max_packet_size
    }
}

async fn run_ota(
    fifocore: FIFOCore,
    bus: u16,
    id: u32,
    payload: Vec<u8>,
    status: Arc<watch::Sender<OtaFlashStatus>>,
) {
    let mut scratch_buf = [0_u8; 64];

    let io = match ClientIO::open(fifocore, bus, id, status.clone()) {
        Ok(io) => io,
        Err(e) => {
            log_error!("[RdxOTA] Failed to open session: {e}");
            let new_state = status.borrow().swap_state(OtaFlashState::Fail, Some(format!("{e}")));
            status.send_replace(new_state);
            return;
        }
    };
    let new_state = status.borrow().swap_state(OtaFlashState::Running, None);
    status.send_replace(new_state);
    let mut runner = RdxOtaClient::new(&payload, &mut scratch_buf, id, io);
    match runner.run().await {
        Ok(()) => {
            let new_state = status.borrow().swap_state(OtaFlashState::Finished, None);
            status.send_replace(new_state);
        }
        Err(e) => {
            log_error!("OTA failed: {e}");
            let new_state = status.borrow().swap_state(OtaFlashState::Fail, Some(format!("{e}")));
            status.send_replace(new_state);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct OtaAddress {
    bus_id: u16,
    device_id: u32,
}

impl OtaAddress {
    pub fn new(bus_id: u16, device_id: u32) -> Self {
        Self { bus_id, device_id }
    }
    pub fn valid(&self) -> bool {
        (self.device_id >> 16 & 0xff) == 0xe
    }

    pub fn parse_path(bus_str: &str, id_str: &str) -> Result<Self, axum::response::Response> {
        let Ok(bus) = u16::from_str_radix(bus_str, 16) else {
            return Err((StatusCode::BAD_REQUEST, "Invalid bus parameter").into_response());
        };
        let Ok(id) = u32::from_str_radix(id_str, 16) else {
            return Err((StatusCode::BAD_REQUEST, "Invalid ID parameter").into_response());
        };
        Ok(Self::new(bus, id))
    }
}

pub(crate) struct OtaTask {
    pub(crate) task: JoinHandle<()>,
    pub(crate) status_send: Arc<watch::Sender<OtaFlashStatus>>,
    pub(crate) status_recv: watch::Receiver<OtaFlashStatus>,
}

impl OtaTask {
    pub fn new(fifocore: FIFOCore, address: OtaAddress, payload: Vec<u8>) -> Self {
        let (status_sender, status_recv) = watch::channel(OtaFlashStatus::default());
        let status_send = Arc::new(status_sender);
        Self {
            task: fifocore.runtime().spawn(run_ota(
                fifocore,
                address.bus_id,
                address.device_id,
                payload,
                status_send.clone(),
            )),
            status_send,
            status_recv: status_recv,
        }
    }

    pub fn abort(&self) {
        self.task.abort();
        self.status_send.send_replace(OtaFlashStatus {
            state: OtaFlashState::Abort,
            written: 0,
            pct_progress: 0.0,
            speed: 0.0,
            error_text: None,
        });
    }
}

impl Drop for OtaTask {
    fn drop(&mut self) {
        self.task.abort();
        self.status_send.send_replace(OtaFlashStatus {
            state: OtaFlashState::Abort,
            written: 0,
            pct_progress: 0.0,
            speed: 0.0,
            error_text: None,
        });
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum OtaFlashState {
    #[default]
    None = 0,
    Running = 1,
    Fail = 2,
    Abort = 3,
    Finished = 4,
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Default, serde::Serialize, serde::Deserialize)]
pub struct OtaFlashStatus {
    /// flashing state
    state: OtaFlashState,
    /// bytes written
    written: usize,
    /// percent progress
    pct_progress: f64,
    /// speed (bytes/s)
    speed: f64,
    /// error text
    error_text: Option<String>,
}

impl OtaFlashStatus {
    pub fn swap_state(&self, new_state: OtaFlashState, error_text: Option<String>) -> Self {
        Self {
            state: new_state,
            written: self.written,
            pct_progress: self.pct_progress,
            speed: self.speed,
            error_text,
        }
    }
}

/// ------- Web server endpoints

pub(crate) async fn ota_start_handler(
    State(state): State<AppState>,
    Path((bus_str, id_str)): Path<(String, String)>,
    body: axum::body::Bytes,
) -> axum::response::Response {
    let addr = match OtaAddress::parse_path(&bus_str, &id_str) {
        Ok(a) => a,
        Err(e) => {
            return e;
        }
    };

    if !addr.valid() {
        return (StatusCode::BAD_REQUEST, "-_-").into_response();
    }
    let mut ota_clients = state.ota_clients.lock();
    ota_clients.insert(addr, OtaTask::new(state.fifocore, addr, body.to_vec()));
    (StatusCode::OK, ":3c").into_response()
}

pub(crate) async fn ota_status_handler(
    State(state): State<AppState>,
    Path((bus_str, id_str)): Path<(String, String)>,
) -> axum::response::Response {
    let addr = match OtaAddress::parse_path(&bus_str, &id_str) {
        Ok(a) => a,
        Err(e) => {
            return e;
        }
    };

    let json = state.ota_clients.lock().get(&addr).map(|inst| {
        inst.status_recv.borrow().clone()
    }).unwrap_or_default();

    let mut response = (StatusCode::OK, axum::Json(json)).into_response();
    response
        .headers_mut()
        .insert("Content-Type", HeaderValue::from_static("application/json"));
    response
}

pub(crate) async fn ota_abort_handler(
    State(state): State<AppState>,
    Path((bus_str, id_str)): Path<(String, String)>,
) -> axum::response::Response {
    let addr = match OtaAddress::parse_path(&bus_str, &id_str) {
        Ok(a) => a,
        Err(e) => {
            return e;
        }
    };
    match state.ota_clients.lock().remove(&addr) {
        Some(inst) => {
            inst.abort();
            (StatusCode::OK, ">w<")
        }
        None => (StatusCode::OK, "-w-"),
    }
    .into_response()
}
