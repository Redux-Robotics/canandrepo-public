//! Client library for the RdxOTA transport protocol.
#![no_std]

use core::{future::Future, time::Duration};
use rdxota_protocol::*;

mod v1;
mod v2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdxOtaIOError {
    RecvTimeout,
    SendTimeout,
    Cancelled,
    Other(&'static str),
}

impl From<RdxOtaIOError> for RdxOtaClientError {
    fn from(value: RdxOtaIOError) -> Self {
        match value {
            RdxOtaIOError::RecvTimeout => Self::RecvTimeout,
            RdxOtaIOError::SendTimeout => Self::SendTimeout,
            RdxOtaIOError::Cancelled => Self::Cancelled,
            RdxOtaIOError::Other(o) => Self::IOError(o),
        }
    }
}
impl core::fmt::Display for RdxOtaIOError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RdxOtaIOError::RecvTimeout => write!(f, "Recieve timeout"),
            RdxOtaIOError::SendTimeout => write!(f, "Send timeout"),
            RdxOtaIOError::Cancelled => write!(f, "Operation cancelled/aborted"),
            RdxOtaIOError::Other(o) => write!(f, "I/O error: {}", o),
        }
    }
}
impl core::error::Error for RdxOtaIOError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RdxOtaClientError {
    RecvTimeout,
    SendTimeout,
    Cancelled,
    IOError(&'static str),
    VersionCheckFail,
    V1Error,
    V2InvalidResponse([u8; 8]),
    V2UnexpectedResponse(rdxota_protocol::otav2::Response),
    V2Nack(rdxota_protocol::otav2::Nack),
    V2UnexpectedAck(rdxota_protocol::otav2::Ack),
    V2InvalidSlot(u16),
    V2FirmwareSlotNotWritable,
    V2CouldNotSwitchToDFU,
    V2Stalled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RdxOtaVersion {
    V1,
    V2,
    Unsupported(u8),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ControlMessage {
    pub data: [u8; 8],
    pub length: u8,
}

impl ControlMessage {
    pub fn new(data: &[u8]) -> Self {
        let mut dest = [0u8; 8];
        let len = data.len().min(dest.len());
        dest[..len].copy_from_slice(data);
        Self {
            data: dest,
            length: len as u8,
        }
    }
}

pub trait RdxOtaClientIO: Send {
    /// Send a message to the message layer.
    fn send(
        &mut self,
        id: u32,
        msg: ControlMessage,
        timeout: core::time::Duration,
    ) -> impl Future<Output = Result<(), RdxOtaIOError>> + Send;
    /// Send an arbitrarily sized message to the message layer. This can be up to [`RdxOtaClient::scratch_buf`] in size.
    fn send_data(
        &mut self,
        id: u32,
        msg: &[u8],
        timeout: core::time::Duration,
    ) -> impl Future<Output = Result<(), RdxOtaIOError>> + Send;
    /// Receive a message from the under layer. It must be address to the device and have an id matching id_to_host().
    fn recv(
        &mut self,
        timeout: core::time::Duration,
    ) -> impl Future<Output = Result<ControlMessage, RdxOtaIOError>> + Send;
    /// Sleep implementation. Included as a result to allow for graceful interruption.
    fn sleep(
        &mut self,
        timeout: core::time::Duration,
    ) -> impl Future<Output = Result<(), RdxOtaIOError>> + Send;
    /// Reset the message layer and clear all buffers.
    fn reset(&mut self);
    /// Update progress
    fn update_progress(
        &mut self,
        written: usize,
        pct_progress: f32,
        speed: f32,
    ) -> impl Future<Output = ()> + Send;
    /// Current monotonic time in seconds.
    fn now_secs(&self) -> f32;
    /// The maximum transport size of the IO layer.
    fn transport_size(&self) -> usize;
}

pub struct RdxOtaClient<'a, 'b, IO: RdxOtaClientIO> {
    payload: &'a [u8],
    scratch_buf: &'b mut [u8],
    id: u32,
    io: IO,
}

impl<'a, 'b, IO: RdxOtaClientIO> RdxOtaClient<'a, 'b, IO> {
    pub fn new(payload: &'a [u8], scratch_buf: &'b mut [u8], id: u32, io: IO) -> Self {
        Self {
            payload,
            scratch_buf,
            id,
            io,
        }
    }

    #[allow(unused)]
    async fn ensure_is_send(
        &'a mut self,
    ) -> impl Future<Output = Result<(), RdxOtaClientError>> + Send + use<'a, 'b, IO> {
        self.run()
    }

    pub async fn run(&mut self) -> Result<(), RdxOtaClientError> {
        log::info!(target: "redux-canlink", "Begin OTA fw update for devtype {} devid {}", (self.id >> 24) & 0x1f, (self.id & 0x3f));
        log::info!(target: "redux-canlink", "Check OTA protocol version...");
        self.io.reset();
        self.io
            .send(
                self.id_to_device(),
                ControlMessage {
                    data: [otav2::index::ctrl::VERSION, 0, 0, 0, 0, 0, 0, 0],
                    length: 8,
                },
                Duration::from_millis(10),
            )
            .await?;

        let msg = self.io.recv(Duration::from_millis(1000)).await?;
        let version = if (msg.data[0] == otav1::index::response::CONTINUE
            && msg.data[1..5] == [0, 0, 0, 0]
            && msg.length == 5)
            || (msg.data[0] == otav1::index::response::ERR && msg.length == 1)
        {
            RdxOtaVersion::V1
        } else if msg.data[0] == otav2::index::ctrl::VERSION {
            if msg.data[1] == otav2::index::OTA_VERSION {
                RdxOtaVersion::V2
            } else {
                RdxOtaVersion::Unsupported(msg.data[1])
            }
        } else {
            RdxOtaVersion::None
        };
        log::info!(target: "redux-canlink", "Detected version as {version:?}");

        match version {
            RdxOtaVersion::V1 => <Self as v1::V1Uploader>::upload(self).await,
            RdxOtaVersion::V2 => <Self as v2::V2Uploader>::upload(self).await,
            RdxOtaVersion::Unsupported(v) => {
                log::info!(target: "redux-canlink", "[redux-canlink] OTA version check failed: recv: version {} is not supported!", v);
                Err(RdxOtaClientError::VersionCheckFail)
            }
            RdxOtaVersion::None => {
                log::info!(target: "redux-canlink", "Could not fetch OTA version from device. Is it connected?");
                Err(RdxOtaClientError::VersionCheckFail)
            }
        }
    }

    pub fn id_to_device(&self) -> u32 {
        self.id | ((OTA_MESSAGE_TO_DEVICE as u32) << 6)
    }

    pub fn id_to_host(&self) -> u32 {
        self.id | ((OTA_MESSAGE_TO_HOST as u32) << 6)
    }

    pub fn id_data(&self) -> u32 {
        self.id | ((OTA_MESSAGE_DATA as u32) << 6)
    }
}

impl core::fmt::Display for RdxOtaClientError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            RdxOtaClientError::RecvTimeout => write!(f, "Message receive timeout"),
            RdxOtaClientError::SendTimeout => write!(f, "Message send timeout"),
            RdxOtaClientError::Cancelled => write!(f, "Operation cancelled/aborted"),
            RdxOtaClientError::IOError(s) => write!(f, "I/O error: {}", s),
            RdxOtaClientError::VersionCheckFail => write!(f, "Version check failed"),
            RdxOtaClientError::V1Error => write!(f, "Device indicated operation failure"),
            RdxOtaClientError::V2InvalidResponse(r) => {
                write!(f, "Invalid response received: {:02x?}", r)
            }
            RdxOtaClientError::V2UnexpectedResponse(response) => {
                write!(f, "Unexpected response received: {:?}", response)
            }
            RdxOtaClientError::V2Nack(nack) => {
                write!(f, "Received NAck: {}", v2::str_for_nack(nack))
            }
            RdxOtaClientError::V2UnexpectedAck(ack) => {
                write!(f, "Received unexpected Ack: {}", v2::str_for_ack(ack))
            }
            RdxOtaClientError::V2InvalidSlot(s) => write!(f, "Invalid file slot: {}", s),
            RdxOtaClientError::V2FirmwareSlotNotWritable => write!(f, "Firmware slot not writable"),
            RdxOtaClientError::V2CouldNotSwitchToDFU => {
                write!(f, "Could not configure device into DFU mode")
            }
            RdxOtaClientError::V2Stalled => write!(f, "Upload progress stalled"),
        }
    }
}
impl core::error::Error for RdxOtaClientError {}
