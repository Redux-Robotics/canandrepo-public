//! Wire types for the CANLink websocket served by ReduxFIFO.
//!
//! These define intermediate structs that have ser/des definitions.
#![cfg_attr(not(feature = "std"), no_std)]

use core::mem::size_of;
use core::mem::size_of_val;

macro_rules! extract_int {
    ($value:ident, $struct:ty, $field:ident, $offset:literal, $int:ty) => {
        <$int>::from_le_bytes(
            $value[$offset..$offset + size_of::<$int>()]
                .try_into()
                .unwrap(),
        )
    };
}

macro_rules! serialize_int {
    ($buffer:ident, $self:ident, $field:ident, $offset:literal) => {
        $buffer[$offset..$offset + size_of_val(&$self.$field)]
            .copy_from_slice(&$self.$field.to_le_bytes())
    };
}

/// Message received from bus.
/// Data size is left off because it's intended to be inferred from the websocket packet length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct CANLinkRxMessage {
    /// 29-bit message ID.
    pub message_id: u32,
    /// The bus ID associated with the message.
    pub bus_id: u16,
    /// Flags (reserved)
    pub flags: u16,
    /// Timestamp in microseconds from the FPGA timebase
    pub timestamp: u64,

    // ===== do not reorder lines above this comment =====
    /// This always holds the largest value.
    /// It's this large for convenience reasons.
    pub data: [u8; 64],
    /// Data size
    pub data_size: usize,
}

impl CANLinkRxMessage {
    const DATA_START: usize = 16;
    /// Data payload as a slice
    pub fn data_slice(&self) -> &[u8] {
        &self.data[..self.data_size.min(64)]
    }

    /// Zeroed buffer the size of this struct as a slice.
    pub const fn buffer() -> [u8; size_of::<Self>()] {
        [0_u8; size_of::<Self>()]
    }

    /// Serialize into an array of the same size.
    /// ```ignore
    /// fn do_thing(msg: &CANLinkRxMessage) {
    ///     let mut buffer = CANLinkRxMessage::buffer();
    ///     let usable_slice: &[u8] = msg.serialize_into(&mut buffer);
    ///     transmit(usable_slice);
    /// }
    /// ```
    pub fn serialize_into<'a>(&self, buffer: &'a mut [u8; size_of::<Self>()]) -> &'a [u8] {
        serialize_int!(buffer, self, message_id, 0);
        serialize_int!(buffer, self, bus_id, 4);
        serialize_int!(buffer, self, flags, 6);
        serialize_int!(buffer, self, timestamp, 8);
        let data_size = self.data_slice().len();
        buffer[Self::DATA_START..Self::DATA_START + data_size].copy_from_slice(self.data_slice());

        &buffer[..Self::DATA_START + self.data_size]
    }
}

#[cfg(feature = "std")]
impl From<CANLinkRxMessage> for Vec<u8> {
    fn from(value: CANLinkRxMessage) -> Self {
        let mut buf = vec![0_u8; CANLinkRxMessage::DATA_START + value.data_size];
        serialize_int!(buf, value, message_id, 0);
        serialize_int!(buf, value, bus_id, 4);
        serialize_int!(buf, value, flags, 6);
        serialize_int!(buf, value, timestamp, 8);
        let data_size = value.data_slice().len();
        buf[CANLinkRxMessage::DATA_START..CANLinkRxMessage::DATA_START + data_size]
            .copy_from_slice(value.data_slice());
        buf
    }
}

impl TryFrom<&[u8]> for CANLinkRxMessage {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::DATA_START {
            return Err(());
        }
        let data_size = (value.len() - Self::DATA_START).min(64);
        let mut data = [0_u8; 64];
        data[..data_size].copy_from_slice(&value[Self::DATA_START..Self::DATA_START + data_size]);

        Ok(Self {
            message_id: extract_int!(value, Self, message_id, 0, u32),
            bus_id: extract_int!(value, Self, bus_id, 4, u16),
            flags: extract_int!(value, Self, flags, 6, u16),
            timestamp: extract_int!(value, Self, timestamp, 8, u64),
            data,
            data_size,
        })
    }
}

/// Message sent to CANLink to be sent onto bus.
#[derive(Debug, Clone, PartialEq, Eq)]
#[repr(C)]
pub struct CANLinkTxMessage {
    /// 29-bit message ID.
    pub message_id: u32,
    /// The bus ID associated with the message.
    pub bus_id: u16,
    /// Flags (reserved)
    pub flags: u16,
    /// This always holds the largest value.
    /// It's this large for convenience reasons/not having to deal with slice ownership
    pub data: [u8; 64],
    /// Data size
    pub data_size: usize,
}

impl CANLinkTxMessage {
    const DATA_START: usize = 8;
    /// Data slice.
    pub fn data_slice(&self) -> &[u8] {
        &self.data[..self.data_size.min(64)]
    }

    /// Zeroed buffer the size of this struct as a slice.
    pub const fn buffer() -> [u8; size_of::<Self>()] {
        [0_u8; size_of::<Self>()]
    }

    /// Serialize into a buffer of the same size.
    ///
    /// ```ignore
    /// fn do_thing(msg: &CANLinkTxMessage) {
    ///     let mut buffer = CANLinkTxMessage::buffer();
    ///     let usable_slice: &[u8] = msg.serialize_into(&mut buffer);
    ///     transmit(usable_slice);
    /// }
    /// ```
    pub fn serialize_into<'a>(&self, buffer: &'a mut [u8; size_of::<Self>()]) -> &'a [u8] {
        serialize_int!(buffer, self, message_id, 0);
        serialize_int!(buffer, self, bus_id, 4);
        serialize_int!(buffer, self, flags, 6);
        buffer[Self::DATA_START..Self::DATA_START + self.data_size]
            .copy_from_slice(self.data_slice());

        &buffer[..Self::DATA_START + self.data_size]
    }
}

#[cfg(feature = "std")]
impl From<CANLinkTxMessage> for Vec<u8> {
    fn from(value: CANLinkTxMessage) -> Self {
        let mut buf = vec![0_u8; 8 + value.data_size];
        serialize_int!(buf, value, message_id, 0);
        serialize_int!(buf, value, bus_id, 4);
        serialize_int!(buf, value, flags, 6);
        let data_size = value.data_slice().len();
        buf[CANLinkTxMessage::DATA_START..CANLinkTxMessage::DATA_START + data_size]
            .copy_from_slice(value.data_slice());
        buf
    }
}

impl TryFrom<&[u8]> for CANLinkTxMessage {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        if value.len() < Self::DATA_START {
            return Err(());
        }
        let data_size = (value.len() - Self::DATA_START).min(64);
        let mut data = [0_u8; 64];
        data[..data_size].copy_from_slice(&value[Self::DATA_START..Self::DATA_START + data_size]);

        Ok(Self {
            message_id: extract_int!(value, Self, message_id, 0, u32),
            bus_id: extract_int!(value, Self, bus_id, 4, u16),
            flags: extract_int!(value, Self, flags, 6, u16),
            data,
            data_size,
        })
    }
}
