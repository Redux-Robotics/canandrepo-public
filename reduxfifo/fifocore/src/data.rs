use std::time::Duration;

use rdxota_client::ControlMessage;

use crate::error::Error;

/// Struct indicating the version of ReduxFIFO
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ReduxFIFOVersion {
    /// Season year
    pub year: u16,
    /// Minor version
    pub minor: u8,
    /// Patch version
    pub patch: u8,
}

impl ReduxFIFOVersion {
    pub const fn version() -> Self {
        let year = const_str::parse!(env!("CARGO_PKG_VERSION_MAJOR"), u16);
        let minor = const_str::parse!(env!("CARGO_PKG_VERSION_MINOR"), u8);
        let patch = const_str::parse!(env!("CARGO_PKG_VERSION_PATCH"), u8);
        Self { year, minor, patch }
    }

    pub const fn serialized(&self) -> u32 {
        ((self.year as u32) << 16) | ((self.minor as u32) << 8) | (self.patch as u32)
    }
}

/// Message struct.
#[derive(Clone, Copy, PartialEq, Eq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C, align(4))]
pub struct ReduxFIFOMessage {
    /// 29-bit message ID. This is typically a CAN message ID.
    pub message_id: u32,
    /// The bus ID associated with the message.
    ///
    /// This may not necessarily be a CAN bus. It could be a USB connection, a web connection, or some other backend.
    pub bus_id: u16,

    /// Padding byte
    pub flags: u8,

    /// Valid data size in bytes.
    /// Some buses may only allow specific sizes of data.
    pub data_size: u8,
    /// Timestamp in microseconds, synchronized to some time base.
    /// On the roboRIO this will be to the FPGA time, on other platforms it will typically be CLOCK_MONOTONIC
    pub timestamp: u64,
    /// Message data in bytes.
    pub data: [u8; 64],
}

impl Default for ReduxFIFOMessage {
    fn default() -> Self {
        Self {
            message_id: Default::default(),
            bus_id: Default::default(),
            flags: Default::default(),
            data_size: Default::default(),
            timestamp: Default::default(),
            data: [0u8; 64],
        }
    }
}

impl core::fmt::Debug for ReduxFIFOMessage {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "ReduxFIFOMessage {{ message_id: {:08x}, bus_id: {}, ts: {:.06}, dlc: {}, data: {:02x?} }}",
            self.message_id,
            self.bus_id,
            Duration::from_micros(self.timestamp).as_secs_f64(),
            self.data_size,
            self.data_slice(),
        )
    }
}

impl ReduxFIFOMessage {
    /// Set in the flags field if the message should not enable BRS, or had BRS disabled on a CAN-FD capable bus.
    pub const FLAG_NO_BRS: u8 = 0x1;
    /// Set in the flags field if the message should not be sent as an FD message, or was received as not an FD message on an FD bus.
    pub const FLAG_NO_FD: u8 = 0x2;
    /// Set in the flags field if the message is directly addressed to a device. Only applicable on RdxUsb devices.
    pub const FLAG_DEV: u8 = 0x4;
    /// Set in the flags field if the message is sent from ReduxFIFO.
    pub const FLAG_TX: u8 = 0x8;

    /// Construct a new message from the component bits.
    pub const fn id_data(bus_id: u16, message_id: u32, data: [u8; 64], dlc: u8, flags: u8) -> Self {
        let dlc = if dlc > 64 { 64 } else { dlc };
        Self {
            message_id,
            bus_id,
            data_size: dlc,
            flags,
            timestamp: 0u64,
            data,
        }
    }

    pub const fn id(&self) -> u32 {
        self.message_id & 0x1fff_ffff
    }

    pub const fn rtr(&self) -> bool {
        self.message_id & MessageIdBuilder::ID_FLAG_RTR != 0
    }

    pub const fn err(&self) -> bool {
        self.message_id & MessageIdBuilder::ID_FLAG_ERR != 0
    }

    pub const fn short_id(&self) -> bool {
        self.message_id & MessageIdBuilder::ID_FLAG_11BIT != 0
    }

    pub const fn no_brs(&self) -> bool {
        self.flags & Self::FLAG_NO_BRS != 0
    }

    pub const fn no_fd(&self) -> bool {
        self.flags & Self::FLAG_NO_FD != 0
    }

    pub const fn device(&self) -> bool {
        self.flags & Self::FLAG_DEV != 0
    }

    pub const fn tx(&self) -> bool {
        self.flags & Self::FLAG_TX != 0
    }

    pub fn data_slice(&self) -> &[u8] {
        let data_size = (self.data_size as usize).min(64);
        &self.data[..data_size]
    }
}

#[cfg(feature = "canandmessage")]
impl canandmessage::CanandMessage<ReduxFIFOMessage> for ReduxFIFOMessage {
    fn get_data(&self) -> &[u8] {
        self.data_slice()
    }
    fn get_len(&self) -> u8 {
        self.data_slice().len() as u8
    }
    fn get_id(&self) -> u32 {
        self.id()
    }
    fn try_from_data(id: u32, data: &[u8]) -> Result<Self, canandmessage::CanandMessageError> {
        if data.len() > 64 {
            return Err(canandmessage::CanandMessageError::DataTooLarge(data.len()));
        }
        let mut new_data = [0u8; 64];
        new_data[..data.len()].copy_from_slice(data);
        Ok(Self::id_data(0, id, new_data, data.len() as u8, 0))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct MessageIdBuilder(u32);
impl MessageIdBuilder {
    /// Set in the ID field if the message an error frame.
    pub const ID_FLAG_ERR: u32 = 0x2000_0000;
    /// Set in the ID field if the message is of the 11-bit CAN 2.0A format.
    pub const ID_FLAG_11BIT: u32 = 0x4000_0000;
    /// Set in the ID field if the message is an RTR frame.
    pub const ID_FLAG_RTR: u32 = 0x8000_0000;

    pub const fn new(id: u32) -> Self {
        Self(id & 0x1fff_ffff)
    }

    pub const fn err(self, is_err: bool) -> Self {
        if is_err {
            Self(self.0 | Self::ID_FLAG_ERR)
        } else {
            Self(self.0 & (!Self::ID_FLAG_ERR))
        }
    }

    pub const fn short_id(self, is_short_id: bool) -> Self {
        if is_short_id {
            Self(self.0 | Self::ID_FLAG_11BIT)
        } else {
            Self(self.0 & (!Self::ID_FLAG_11BIT))
        }
    }

    pub const fn rtr(self, is_rtr: bool) -> Self {
        if is_rtr {
            Self(self.0 | Self::ID_FLAG_RTR)
        } else {
            Self(self.0 & (!Self::ID_FLAG_RTR))
        }
    }

    pub const fn build(self) -> u32 {
        self.0
    }
}

impl From<MessageIdBuilder> for u32 {
    fn from(value: MessageIdBuilder) -> Self {
        value.0
    }
}

/// Represents a session handle.
/// The upper 16 bits are the bus id, while the lower 16 bits are the session id.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct ReduxFIFOSession(pub u64);
impl ReduxFIFOSession {
    /// Constructs a session handle from a handle iD and a bus ID.
    pub const fn from_parts(handle_id: u32, bus_id: u16) -> Self {
        Self(((bus_id as u64) << 32) | (handle_id as u64))
    }

    /// Bus ID.
    pub const fn bus_id(&self) -> u16 {
        (self.0 >> 32) as u16
    }

    /// Session ID.
    pub const fn ses_id(&self) -> u32 {
        (self.0 & 0xffffffff) as u32
    }
}

/// FFI-compatible write buffer struct.
///
/// These just get written out on bus.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C, align(4))]
pub struct ReduxFIFOWriteBuffer {
    /// Bus ID to write messages out onto
    pub bus_id: u32,
    /// The status code of the result.
    pub status: i32,
    /// The number of messages written onto bus (output)
    pub messages_written: u32,
    /// The number of messages in this buffer.
    pub length: u32,
}

/// This is a metadata struct for a buffer that ReduxFIFO acts on.
///
/// Buffers are treated as ringbuffers that when pushed-at-full erase their oldest entry.
#[derive(Debug, PartialEq, Eq, Clone)]
#[repr(C, align(4))]
pub struct ReduxFIFOReadBuffer {
    /// Session ID associated with this buffer metadata object.
    pub session: ReduxFIFOSession,
    /// The status code of the result.
    pub status: i32,
    /// The next index where the newest message would be written during a read operation.
    /// If valid_length == max_length, then this is the oldest message in the buffer.
    pub next_idx: u32,
    /// The number of valid messages in this buffer.
    /// This is supplied by ReduxFIFO.
    pub valid_length: u32,
    /// The absolute max length of the buffer.
    pub max_length: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
#[non_exhaustive]
pub struct ReduxFIFOSessionConfig {
    pub filter_id: u32,
    pub filter_mask: u32,
    pub echo_tx: bool,
}

impl ReduxFIFOSessionConfig {
    pub fn new(filter_id: u32, filter_mask: u32) -> Self {
        Self {
            filter_id,
            filter_mask,
            echo_tx: false,
        }
    }

    pub const fn message_matches(&self, msg: &ReduxFIFOMessage) -> bool {
        msg.message_id & self.filter_mask == self.filter_id
    }
}

impl Default for ReduxFIFOSessionConfig {
    fn default() -> Self {
        Self {
            filter_id: 0x0e0000,
            filter_mask: 0xff0000,
            echo_tx: false,
        }
    }
}

impl From<ReduxFIFOMessage> for ControlMessage {
    fn from(value: ReduxFIFOMessage) -> Self {
        ControlMessage {
            data: value.data[..8].try_into().unwrap(),
            length: value.data_size,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct ReduxFIFOStatus(pub i32);

impl From<Result<(), Error>> for ReduxFIFOStatus {
    fn from(value: Result<(), Error>) -> Self {
        Self(value.err().map_or(0, i32::from))
    }
}
