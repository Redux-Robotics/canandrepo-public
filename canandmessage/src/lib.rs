#![cfg_attr(
    not(any(
        feature = "alchemist",
        feature = "simulation",
    )),
    no_std
)]
#![allow(unused_imports)]
#![allow(unused_macros)]
extern crate bitvec;
use core::ops;
#[macro_use]
extern crate canandmessage_defn_macro as _;

/// Most of canandmessage is meant for devices with very specific send/receive flows.
/// Client-side infra (e.g. fatp, vendordep, and alchemist) require more generic message parsing,
/// so this module aims to provide more general message interfaces that can be up/downcasted
/// to device-specific structs when needed
pub mod generic;
/// Shared traits that each device can implement
pub mod traits;

#[cfg(feature = "alchemist")]
use canandmessage_alchemist_generation::gen_typescript_utils;

#[cfg(feature = "alchemist")]
use canandmessage_defn_macro::gen_alchemist_utils;

#[cfg(feature = "simulation")]
use canandmessage_defn_macro::gen_simulation_utils;

#[gen_device_messages(src_file = "messages/cananddevice.toml", mode = "both")]
/// Messages for the Cananddevice.
pub mod cananddevice {}

#[cfg(any(feature = "canandmag", feature = "alchemist"))]
#[gen_device_messages(src_file = "messages/canandmag.toml", mode = "both")]
/// Messages for the Canandmag.
pub mod canandmag {}

#[cfg(any(feature = "canandgyro", feature = "alchemist"))]
#[gen_device_messages(src_file = "messages/canandgyro.toml", mode = "both")]
/// Messages for the Canandgyro.
pub mod canandgyro {}

#[cfg(any(feature = "canandcolor", feature = "alchemist"))]
#[gen_device_messages(src_file = "messages/canandcolor.toml", mode = "both")]
/// Messages for the Canandcolor.
pub mod canandcolor {}

/*
 *  ===============================
 *  ALCHEMIST LAND. THERE BE GHOSTS
 *  ===============================
 */

#[cfg(feature = "alchemist")]
#[gen_alchemist_utils(
    src_file = "messages/canandmag.toml",
    src_file = "messages/canandcolor.toml",
    src_file = "messages/canandgyro.toml",
)]
#[cfg(feature = "alchemist")]
pub mod alchemist {
    use crate::canandcolor;
    use crate::canandgyro;
    use crate::canandmag;

    use crate::traits::CanandDeviceMessage;
    use crate::traits::CanandDeviceSetting;
}

#[cfg(feature = "simulation")]
#[gen_simulation_utils(
    src_file = "messages/canandmag.toml",
    src_file = "messages/canandcolor.toml",
    src_file = "messages/canandgyro.toml",
)]
pub mod simulation {
    use crate::canandcolor;
    use crate::canandgyro;
    use crate::canandmag;
}

pub struct CanandMessageWrapper<T: CanandMessage<T>>(pub T);

impl<T: CanandMessage<T>> ops::Deref for CanandMessageWrapper<T> {
    type Target = T; // Our wrapper struct will coerce into Option
    fn deref(&self) -> &T {
        &self.0 // We just extract the inner element
    }
}

pub trait CanandMessage<T> {
    fn get_data(&self) -> &[u8];
    fn get_len(&self) -> u8;
    fn get_id(&self) -> u32;
    fn try_from_data(id: u32, data: &[u8]) -> Result<T, CanandMessageError>;
}

#[cfg_attr(feature = "device", derive(defmt::Format))]
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum CanandMessageError {
    DataTooLarge(usize),
    DataSizeInvalidForFd(usize),
}


impl core::fmt::Display for CanandMessageError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DataTooLarge(value) => write!(f, "Data too large: {value}"),
            Self::DataSizeInvalidForFd(value) => write!(f, "Data invalid for transport: {value}"),
        }
    }
}

impl core::error::Error for CanandMessageError {}

/// Generic can message struct that downstream libraries can convert to packets.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(C)]
pub struct CanMessage {
    pub data: [u8; 8],
    pub id: u32,
    pub len: u8,
}

impl CanandMessage<CanMessage> for CanMessage {
    fn get_data(&self) -> &[u8] {
        &self.data[..self.len as usize]
    }
    fn get_len(&self) -> u8 {
        self.len
    }
    fn get_id(&self) -> u32 {
        self.id
    }
    fn try_from_data(id: u32, data: &[u8]) -> Result<CanMessage, CanandMessageError> {
        if data.len() > 8 {
            return Err(CanandMessageError::DataTooLarge(data.len()));
        }
        let mut new_data = [0u8; 8];
        new_data[..data.len()].copy_from_slice(data);
        Ok(CanMessage {
            data: new_data,
            id,
            len: data.len() as u8,
        })
    }
}

#[allow(unused)]
fn u24_from_le_bytes(data: [u8; 3]) -> u32 {
    (data[0] as u32) | ((data[1] as u32) << 8) | ((data[2] as u32) << 16)
}


#[cfg(feature = "alchemist")]
#[gen_typescript_utils(
    src_file = "messages/canandmag.toml",
    src_file = "messages/canandcolor.toml",
    src_file = "messages/canandgyro.toml",
)]
pub mod typescript_utils {}

pub const REDUX_VENDOR_ID: u8 = 0xe;
