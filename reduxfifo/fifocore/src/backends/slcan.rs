use std::{collections::VecDeque, sync::Arc, time::Duration};

use parking_lot::Mutex;
use tokio::io::{AsyncReadExt as _, AsyncWriteExt as _};

use crate::{
    MessageIdBuilder, ReduxFIFOMessage,
    backends::{Backend, BackendOpen, SessionTable},
    error::Error,
    log_debug, log_error, log_trace,
};

#[derive(Debug)]
pub struct SlcanBackend {
    params: Params,
    tx_queue: tokio::sync::mpsc::Sender<ReduxFIFOMessage>,
    run_task: tokio::task::JoinHandle<()>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Params {
    path: String,
    baud: u32,
}

fn split_once<'a>(s: &'a str, d: &str) -> Result<(&'a str, &'a str), Error> {
    s.split_once(d).ok_or(Error::InvalidBus)
}
impl SlcanBackend {
    fn parse_params(s: &str) -> Result<Params, Error> {
        // slcan:[baud].[path]
        let (backend_type, backend_args) = split_once(s, ":")?;
        if backend_type != "slcan" {
            return Err(Error::BusNotSupported);
        }
        let (baud_str, path) = split_once(backend_args, ":")?;
        let baud = baud_str.parse::<u32>().map_err(|_| Error::InvalidBus)?;

        Ok(Params {
            path: path.to_string(),
            baud,
        })
    }
}

impl Backend for SlcanBackend {
    type State = ();

    fn start_session(
        &mut self,
        _msg_count: u32,
        _config: &crate::ReduxFIFOSessionConfig,
    ) -> Result<Self::State, crate::error::Error> {
        Ok(())
    }

    fn write_single(&mut self, msg: &crate::ReduxFIFOMessage) -> Result<(), crate::error::Error> {
        self.tx_queue
            .try_send(*msg)
            .map_err(|_| Error::BusBufferFull)
    }

    fn params_match(&self, params: &str) -> bool {
        if let Ok(params) = Self::parse_params(params) {
            params == self.params
        } else {
            false
        }
    }

    fn max_packet_size(&self) -> usize {
        8
    }
}

impl BackendOpen for SlcanBackend {
    fn open(
        bus_id: u16,
        params: &str,
        runtime: tokio::runtime::Handle,
        ses_table: Arc<Mutex<SessionTable<Self::State>>>,
    ) -> Result<Self, Error> {
        log_debug!("Attempt to open slcan...");
        let params = Self::parse_params(params)?;
        log_debug!("Params parsed: {params:?}");

        let stream =
            tokio_serial::SerialStream::open(&tokio_serial::new(&params.path, params.baud))
                .map_err(|e| {
                    log_error!(
                        "Failed to open slcan bus {} @ {} baud: {e}",
                        params.path,
                        params.baud
                    );
                    Error::FailedToOpenBus
                })?;

        let (tx_queue_send, tx_queue_recv) = tokio::sync::mpsc::channel(128);

        Ok(Self {
            params: params.clone(),
            tx_queue: tx_queue_send,
            run_task: runtime.spawn(run_backend_wrapper(
                params,
                stream,
                tx_queue_recv,
                bus_id,
                ses_table,
            )),
        })
    }
}

impl Drop for SlcanBackend {
    fn drop(&mut self) {
        self.run_task.abort();
    }
}

enum NextOperation {
    RxData(usize),
    TxMessage(ReduxFIFOMessage),
}

async fn run_backend_wrapper(
    params: Params,
    stream: tokio_serial::SerialStream,
    tx_queue: tokio::sync::mpsc::Receiver<ReduxFIFOMessage>,
    bus_id: u16,
    sessions: Arc<Mutex<SessionTable<()>>>,
) {
    if let Err(e) = run_backend(stream, tx_queue, bus_id, sessions).await {
        log_error!(
            "slcan backend {bus_id}: {} @ {} died: {e}",
            params.path,
            params.baud
        );
    }
}

async fn run_backend(
    mut stream: tokio_serial::SerialStream,
    mut tx_queue: tokio::sync::mpsc::Receiver<ReduxFIFOMessage>,
    bus_id: u16,
    sessions: Arc<Mutex<SessionTable<()>>>,
) -> Result<(), anyhow::Error> {
    log_trace!("slcan: start backend for {bus_id}");
    let mut buf = bytes::BytesMut::with_capacity(1024);
    let mut state = RxStateMachine::new(bus_id);
    let mut tx_buf: Vec<u8> = Vec::with_capacity(32);
    stream.write_all(b"\r\r\rC\r\r\r").await?;
    tokio::time::sleep(Duration::from_millis(20)).await;
    stream.try_read(&mut buf).ok();
    buf.clear();

    stream.write_all(b"S8\r").await?;
    stream.write_all(b"O\r").await?;

    loop {
        buf.clear();
        let next_op = tokio::select! {
            rx = stream.read_buf(&mut buf) => {
                NextOperation::RxData(rx?)
            }
            tx = tx_queue.recv() => {
                let Some(msg) = tx else { return Ok(()); };
                NextOperation::TxMessage(msg)
            }
        };
        match next_op {
            NextOperation::RxData(read_len) => {
                state.ingest(&buf[..read_len]);
                let mut ses_lock = sessions.lock();
                while let Some(mut msg) = state.drain() {
                    msg.timestamp = crate::timebase::now_us() as u64;
                    ses_lock.ingest_message(msg);
                }
                drop(ses_lock);
            }
            NextOperation::TxMessage(msg) => {
                serialize_into(&mut tx_buf, &msg)?;
                stream.write_all(&tx_buf).await?;
            }
        }
    }
}

fn serialize_into(tx_buf: &mut Vec<u8>, msg: &crate::ReduxFIFOMessage) -> anyhow::Result<()> {
    let len = msg.data_slice().len().min(8);
    tx_buf.clear();
    tx_buf.extend_from_slice(format!("T{:08X}{len}", msg.message_id).as_bytes());
    for byte in &msg.data_slice()[..len] {
        tx_buf.extend_from_slice(format!("{byte:02X}").as_bytes());
    }
    tx_buf.push(b'\r');
    Ok(())
}

struct RxStateMachine {
    in_buf: VecDeque<u8>,
    bus_id: u16,
}

const STD_HEADER: usize = 5;
const EXT_HEADER: usize = 10;

impl RxStateMachine {
    pub const fn new(bus_id: u16) -> Self {
        Self {
            in_buf: VecDeque::new(),
            bus_id,
        }
    }
    pub fn ingest(&mut self, data: &[u8]) {
        self.in_buf.extend(data);
    }

    pub fn drain(&mut self) -> Option<ReduxFIFOMessage> {
        if self.in_buf.is_empty() {
            return None;
        }

        loop {
            // read the first character.
            let first = *self.in_buf.front()?;
            match first {
                b't' | b'r' => {
                    // 11-bit id
                    let is_remote = first == b'r';
                    if self.in_buf.len() < STD_HEADER {
                        // len < b'tIIIL'.len()
                        return None;
                    }
                    let id = self
                        .in_buf
                        .iter()
                        .skip(1)
                        .take(3)
                        .map(|b| from_bcx(*b).unwrap_or(0))
                        .fold(0_u32, |prev, next| (prev << 4) | (next as u32))
                        | MessageIdBuilder::ID_FLAG_11BIT;
                    let len = self
                        .in_buf
                        .get(4)
                        .unwrap_or(&b'0')
                        .saturating_sub(b'0')
                        .min(8);
                    return self.conjure_message(id, len, is_remote, STD_HEADER);
                }
                b'T' | b'R' => {
                    // 29-bit id
                    let is_remote = first == b'R';
                    if self.in_buf.len() < EXT_HEADER {
                        // len < b'tIIIIIIIIL'.len()
                        return None;
                    }
                    let id = self
                        .in_buf
                        .iter()
                        .skip(1)
                        .take(8)
                        .map(|b| from_bcx(*b).unwrap_or(0))
                        .fold(0_u32, |prev, next| (prev << 4) | (next as u32));
                    let len = self
                        .in_buf
                        .get(9)
                        .unwrap_or(&b'0')
                        .saturating_sub(b'0')
                        .min(8);
                    return self.conjure_message(id, len, is_remote, EXT_HEADER);
                }
                _ => {
                    // irrelevant garbage
                    self.in_buf.pop_front()?;
                    continue;
                }
            }
        }
    }

    fn conjure_message(
        &mut self,
        id: u32,
        len: u8,
        is_remote: bool,
        header_size: usize,
    ) -> Option<ReduxFIFOMessage> {
        let serialized_len = header_size + len as usize * 2;
        if is_remote {
            let msg = ReduxFIFOMessage::id_data(
                self.bus_id,
                id | MessageIdBuilder::ID_FLAG_RTR,
                [0_u8; _],
                len,
                0,
            );
            drop(self.in_buf.drain(..header_size));
            return Some(msg);
        } else if self.in_buf.len() < serialized_len {
            // not full
            return None;
        } else {
            let mut data = [0_u8; _];
            for i in 0..len as usize {
                let msb = from_bcx(*self.in_buf.get(header_size + i * 2).unwrap_or(&0))
                    .unwrap_or_default();
                let lsb = from_bcx(*self.in_buf.get(header_size + i * 2 + 1).unwrap_or(&0))
                    .unwrap_or_default();
                data[i] = (msb << 4) | lsb;
            }

            let msg = ReduxFIFOMessage::id_data(self.bus_id, id, data, len, 0);
            drop(self.in_buf.drain(..serialized_len));
            return Some(msg);
        }
    }
}
fn from_bcx(a: u8) -> Option<u8> {
    let a_lower = a & 0b1011111;
    if a >= b'0' && a <= b'9' {
        Some(a - b'0')
    } else if a_lower >= b'A' && a_lower <= b'F' {
        Some(a_lower - b'A' + 10u8)
    } else {
        None
    }
}
