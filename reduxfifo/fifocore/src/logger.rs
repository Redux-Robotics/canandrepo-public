use crate::ReduxFIFOMessage;
use tokio::{fs::OpenOptions, io::AsyncWriteExt, runtime::Handle, task::JoinHandle};

pub type LoggerTx = Option<tokio::sync::mpsc::Sender<ReduxFIFOMessage>>;

#[derive(Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct LogHeader {
    /// 29-bit message ID. This is typically a CAN message ID.
    pub message_id: u32,
    /// The bus ID associated with the message.
    ///
    /// This may not necessarily be a CAN bus. It could be a USB connection, a web connection, or some other backend.
    pub bus_id: u16,

    /// Misc flags
    pub flags: u8,

    /// Valid data size in bytes.
    /// Some buses may only allow specific sizes of data.
    pub data_size: u8,
    /// Timestamp in microseconds, synchronized to some time base.
    /// On the roboRIO this will be to the FPGA time, on other platforms it will typically be CLOCK_MONOTONIC
    pub timestamp: u64,
}

impl From<ReduxFIFOMessage> for LogHeader {
    fn from(value: ReduxFIFOMessage) -> Self {
        Self {
            message_id: value.message_id,
            bus_id: value.bus_id,
            flags: value.flags,
            data_size: value.data_size,
            timestamp: value.timestamp,
        }
    }
}

macro_rules! log_err_and_bail {
    ($e:expr, $fname:expr) => {{
        match $e {
            Ok(o) => o,
            Err(e) => {
                crate::log_error!("Open log file {} failed: {e}", $fname.display());
                return;
            }
        }
    }};
}

#[derive(Debug)]
pub struct Logger {
    task: JoinHandle<()>,
    tx: tokio::sync::mpsc::Sender<ReduxFIFOMessage>,
}

impl Logger {
    pub fn new(fname: std::path::PathBuf, runtime: Handle) -> Self {
        let (sender, receiver) = tokio::sync::mpsc::channel(128);
        Self {
            task: runtime.spawn(logger_task(fname, receiver)),
            tx: sender,
        }
    }

    pub fn sender(&self) -> LoggerTx {
        Some(self.tx.clone())
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn logger_task(
    fname: std::path::PathBuf,
    mut rx: tokio::sync::mpsc::Receiver<ReduxFIFOMessage>,
) {
    crate::log_info!("Opening log file {}", fname.display());
    let mut file = log_err_and_bail!(
        OpenOptions::new()
            .append(true)
            .create(true)
            .open(&fname)
            .await,
        fname
    );
    log_err_and_bail!(file.write_all(b"ReduxFIFOLogFile").await, fname);
    let mut buffer = Vec::with_capacity(80);

    while let Some(msg) = rx.recv().await {
        buffer.clear();
        let header = LogHeader::from(msg);
        buffer.extend_from_slice(bytemuck::bytes_of(&header));
        buffer.extend_from_slice(msg.data_slice());
        if let Err(e) = file.write_all(&buffer).await {
            crate::log_error!("Failed write to {}: {e}", fname.display());
            break;
        }
    }

    rx.close();

    crate::log_info!("Closing log file {}", fname.display());
    file.shutdown().await.ok();
}
