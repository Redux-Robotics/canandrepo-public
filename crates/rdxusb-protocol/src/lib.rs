#![no_std]

use bytemuck::{Pod, Zeroable};

/// In bulk xfer endpoint (has top bit set)
pub const DEFAULT_ENDPOINT_IN: u8 = 0x81;
/// Out bulk xfer endpoint
pub const DEFAULT_ENDPOINT_OUT: u8 = 0x02;

/// this bit is true on arbitration IDs [`RdxUsbFsPacket::arb_id`] that are extended (29-bit).
pub const MESSAGE_ARB_ID_EXT: u32 = 0x80000000;
/// this bit is true on arbitration IDs [`RdxUsbFsPacket::arb_id`] associated with an RTR frame.
pub const MESSAGE_ARB_ID_RTR: u32 = 0x40000000;
/// Specifies the frame is specifically addressed to/from the device.
///
/// For messages from device to host, this means that the message in fact originates from the device,
/// and not any connected devices proxied through other buses.
///
/// For messages from host to device, the device will understand that the host message is meant for it,
/// regardless of any configured device id bits.
pub const MESSAGE_ARB_ID_DEVICE: u32 = 0x20000000;

/// Generic data packet passed to/from RdxUsb APIs.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Pod, Zeroable)]
#[repr(C, packed)]
pub struct RdxUsbPacket {
    /// Message id (CAN arbitration id)
    pub message_id: u32,
    /// Relevant channel. Zero most of the time.
    pub channel: u16,
    /// Padding byte
    pub reserved: u8,
    /// Valid data size in bytes.
    pub data_size: u8,
    /// Timestamp since boot (nanoseconds)
    pub timestamp_ns: u64,
    /// data (max size: 64 bytes)
    pub data: [u8; 64],
}

impl RdxUsbPacket {
    pub const fn new(
        message_id: u32,
        channel: u16,
        data: [u8; 64],
        data_size: u8,
        timestamp_ns: u64,
    ) -> Self {
        Self {
            message_id,
            channel,
            reserved: 0,
            data_size: if data_size <= 64 { data_size } else { 64 },
            timestamp_ns,
            data,
        }
    }

    pub fn wire_length(&self) -> usize {
        (16 + self.data_size as usize).min(80)
    }

    pub fn encode(&self) -> &[u8; Self::SIZE] {
        bytemuck::cast_ref(self)
    }

    pub fn from_buf(buf: &[u8; Self::SIZE]) -> &Self {
        bytemuck::cast_ref(buf)
    }

    pub fn from_slice(buf: &[u8]) -> Option<(Self, usize)> {
        let len = buf.len();
        if len < 16 {
            return None;
        }
        let packet_len = buf[7] as usize + 16;
        if len >= packet_len {
            let mut data = [0_u8; 80];
            data[..packet_len].copy_from_slice(&buf[..packet_len]);
            Some((bytemuck::cast(data), packet_len))
        } else {
            None
        }
    }

    /// The message arbitration id
    pub const fn id(&self) -> u32 {
        self.message_id & 0x1fff_ffff
    }

    /// Does the packet use extended (29-bit) IDs?
    pub const fn extended(&self) -> bool {
        self.message_id & MESSAGE_ARB_ID_EXT != 0
    }

    /// Is the packet an RTR packet?
    pub const fn rtr(&self) -> bool {
        self.message_id & MESSAGE_ARB_ID_RTR != 0
    }

    /// Is the packet a device packet?
    pub const fn device(&self) -> bool {
        self.message_id & MESSAGE_ARB_ID_DEVICE != 0
    }

    /// Should always be 80.
    pub const SIZE: usize = core::mem::size_of::<Self>();
}

/// Struct returned by the device info control request
#[derive(Debug, PartialEq, Eq, Clone, Copy, Pod, Zeroable)]
#[repr(C, packed)]
pub struct RdxUsbDeviceInfo {
    /// The SKU index of the device (the first number in the serial)
    pub sku: u16,
    /// The interface index that the RdxUSB interface uses
    pub interface_idx: u8,
    /// The number of channels that the RdxUSB interface supports (0-indexed)
    pub n_channels: u8,
    /// The major protocol version
    pub protocol_version_major: u16,
    /// The minor protocol version
    pub protocol_version_minor: u16,
    /// Reserved bits
    pub reserved: [u8; 24],
}

impl RdxUsbDeviceInfo {
    /// Should always be 32.
    pub const SIZE: usize = core::mem::size_of::<Self>();

    pub fn encode(&self) -> &[u8; Self::SIZE] {
        bytemuck::cast_ref(self)
    }

    pub fn from_buf(buf: [u8; Self::SIZE]) -> Self {
        bytemuck::cast(buf)
    }
}

/// Control requests supported
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u8)]
pub enum RdxUsbCtrl {
    DeviceInfo = 0,
}

/// USB protocol version 2
pub const PROTOCOL_VERSION_MAJOR_FS: u16 = 2;
