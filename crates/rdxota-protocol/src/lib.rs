#![no_std]
/// common protocol crate for rdxota protocols
/// intended to be canonical.

/// Data index (bidir)
pub const OTA_MESSAGE_DATA: u8 = 0x8;
/// To host index (bidir)
pub const OTA_MESSAGE_TO_HOST: u8 = 0x9;
/// To device index (bidir)
pub const OTA_MESSAGE_TO_DEVICE: u8 = 0xA;

/// OTAv2 indexes
pub mod otav1;

/// OTAv2 indexes
pub mod otav2;
