use super::{api_index_match, build_frc_can_id, MessageCastError, WrapperSerializable};
use crate::traits::CanandDeviceMessage;

macro_rules! impl_64bit_msg {
    ($msg:ident, $field:ident, $min_dlc:expr) => {
        #[cfg_attr(feature = "device", derive(defmt::Format))]
        #[derive(Copy, Clone, PartialEq, Eq, Debug)]
        pub struct $msg(pub [u8; 8]);

        impl $msg {
            pub const fn new(value: [u8; 8]) -> Self {
                Self(value)
            }
        }

        impl WrapperSerializable for $msg {
            fn try_from_wrapper<T: crate::CanandMessage<T>>(
                cmsg: &crate::CanandMessageWrapper<T>,
            ) -> Result<Self, MessageCastError> {
                let id = cmsg.get_id();
                if !api_index_match(id, crate::cananddevice::MessageIndex::$msg.into()) {
                    return Err(MessageCastError::WrongMessage(((id >> 6) & 0xff) as u8));
                }

                #[allow(unused_comparisons)]
                if cmsg.get_len() < $min_dlc {
                    return Err(MessageCastError::WrongDlc(cmsg.get_len()));
                }
                Ok(Self(cmsg.get_data()[0..8].try_into().unwrap()))
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
                        crate::cananddevice::MessageIndex::$msg as u16,
                        device_id,
                    ),
                    &data,
                )?))
            }
        }

        impl From<[u8; 8]> for $msg {
            fn from(value: [u8; 8]) -> Self {
                Self(value)
            }
        }

        impl From<$msg> for [u8; 8] {
            fn from(value: $msg) -> Self {
                value.0
            }
        }
    };
}

macro_rules! impl_64bit_msg_for_device {
    ($dev:ident, $msg:ident, $field:ident) => {
        impl TryFrom<crate::$dev::Message> for $msg {
            type Error = MessageCastError;

            fn try_from(value: crate::$dev::Message) -> Result<Self, Self::Error> {
                match value {
                    crate::$dev::Message::$msg { $field } => Ok(Self($field)),
                    _ => Err(MessageCastError::WrongMessage(value.raw_message_index())),
                }
            }
        }

        impl From<$msg> for crate::$dev::Message {
            fn from(value: $msg) -> Self {
                crate::$dev::Message::$msg { $field: value.0 }
            }
        }
    };
}

impl_64bit_msg!(CanIdArbitrate, addr_value, 8);
impl CanIdArbitrate {
    pub const fn arbitrate_all() -> Self {
        Self([0xffu8; 8])
    }
}
impl_64bit_msg!(CanIdError, addr_value, 8);
impl_64bit_msg!(OtaData, data, 0);
impl_64bit_msg!(OtaToHost, to_host_data, 0);
impl_64bit_msg!(OtaToDevice, to_device_data, 0);

mod cananddevice {
    use super::*;
    impl_64bit_msg_for_device!(cananddevice, CanIdArbitrate, addr_value);
    impl_64bit_msg_for_device!(cananddevice, CanIdError, addr_value);
    impl_64bit_msg_for_device!(cananddevice, OtaData, data);
    impl_64bit_msg_for_device!(cananddevice, OtaToHost, to_host_data);
}

#[cfg(feature = "canandmag")]
mod canandmag {
    use super::*;
    impl_64bit_msg_for_device!(canandmag, CanIdArbitrate, addr_value);
    impl_64bit_msg_for_device!(canandmag, CanIdError, addr_value);
    impl_64bit_msg_for_device!(canandmag, OtaData, data);
    impl_64bit_msg_for_device!(canandmag, OtaToHost, to_host_data);
}

#[cfg(feature = "canandgyro")]
mod canandgyro {
    use super::*;
    impl_64bit_msg_for_device!(canandgyro, CanIdArbitrate, addr_value);
    impl_64bit_msg_for_device!(canandgyro, CanIdError, addr_value);
    impl_64bit_msg_for_device!(canandgyro, OtaData, data);
    impl_64bit_msg_for_device!(canandgyro, OtaToHost, to_host_data);
}

#[cfg(feature = "canandcolor")]
mod canandcolor {
    use super::*;
    impl_64bit_msg_for_device!(canandcolor, CanIdArbitrate, addr_value);
    impl_64bit_msg_for_device!(canandcolor, CanIdError, addr_value);
    impl_64bit_msg_for_device!(canandcolor, OtaData, data);
    impl_64bit_msg_for_device!(canandcolor, OtaToHost, to_host_data);
}
