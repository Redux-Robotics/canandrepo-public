use super::{api_index_match, build_frc_can_id, MessageCastError, WrapperSerializable};
use crate::{cananddevice, traits::CanandDeviceMessage};

#[cfg_attr(feature = "device", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Enumerate {
    pub serial: [u8; 6],
    pub is_bootloader: bool,
    pub reserved: u16,
}

impl Enumerate {
    pub const fn new(serial: [u8; 6], is_bootloader: bool, reserved: u16) -> Self {
        Self {
            serial,
            is_bootloader,
            reserved,
        }
    }
}

impl WrapperSerializable for Enumerate {
    fn try_from_wrapper<T: crate::CanandMessage<T>>(
        cmsg: &crate::CanandMessageWrapper<T>,
    ) -> Result<Self, MessageCastError> {
        let id = cmsg.get_id();
        if !api_index_match(id, cananddevice::MessageIndex::Enumerate.into()) {
            return Err(MessageCastError::WrongMessage(((id >> 6) & 0xff) as u8));
        }

        if cmsg.get_len() < 8 {
            return Err(MessageCastError::WrongDlc(cmsg.get_len()));
        }
        let data: [u8; 8] = cmsg.get_data()[0..8].try_into().unwrap();
        Ok(data.into())
    }

    fn try_into_wrapper<T: crate::CanandMessage<T>>(
        &self,
        device_type: u8,
        device_id: u8,
    ) -> Result<crate::CanandMessageWrapper<T>, crate::CanandMessageError> {
        let data: [u8; 8] = (*self).into();
        Ok(crate::CanandMessageWrapper(T::try_from_data(
            build_frc_can_id(
                device_type,
                crate::REDUX_VENDOR_ID,
                cananddevice::MessageIndex::Enumerate as u16,
                device_id,
            ),
            &data,
        )?))
    }
}

impl From<[u8; 8]> for Enumerate {
    fn from(value: [u8; 8]) -> Self {
        Self {
            serial: value[..6].try_into().unwrap(),
            is_bootloader: (value[6] & 0b1) != 0,
            reserved: u16::from_le_bytes(value[6..].try_into().unwrap()) >> 1,
        }
    }
}

impl From<Enumerate> for [u8; 8] {
    fn from(value: Enumerate) -> Self {
        let mut data = [0u8; 8];
        data[..6].copy_from_slice(&value.serial[..6]);
        data[6] = (value.reserved << 1) as u8 | (value.is_bootloader as u8);
        data[7] = (value.reserved >> 9) as u8;
        data
    }
}

macro_rules! impl_conv {
    ($dev:ident) => {
        impl TryFrom<crate::$dev::Message> for Enumerate {
            type Error = MessageCastError;

            fn try_from(value: crate::$dev::Message) -> Result<Self, Self::Error> {
                match value {
                    crate::$dev::Message::Enumerate {
                        serial,
                        is_bootloader,
                    } => Ok(Self {
                        serial,
                        is_bootloader,
                        reserved: 0,
                    }),
                    _ => Err(MessageCastError::WrongMessage(value.raw_message_index())),
                }
            }
        }

        impl From<Enumerate> for crate::$dev::Message {
            fn from(value: Enumerate) -> Self {
                crate::$dev::Message::Enumerate {
                    serial: value.serial,
                    is_bootloader: value.is_bootloader,
                }
            }
        }
    };
}

#[cfg(feature = "canandmag")]
impl_conv!(canandmag);

#[cfg(feature = "canandgyro")]
impl_conv!(canandgyro);

#[cfg(feature = "canandcolor")]
impl_conv!(canandcolor);
