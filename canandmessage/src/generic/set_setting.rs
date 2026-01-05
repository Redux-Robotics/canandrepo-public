use super::{
    api_index_match, build_frc_can_id, MessageCastError, SettingCastError, WrapperSerializable,
};
use crate::{
    cananddevice,
    traits::{CanandDeviceMessage, CanandDeviceSetting},
    CanandMessageError,
};

#[cfg_attr(feature = "device", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct SetSetting {
    pub index: u8,
    pub value: [u8; 6],
    pub flags: crate::cananddevice::types::SettingFlags,
}

impl SetSetting {
    pub const fn new(
        index: u8,
        value: [u8; 6],
        flags: crate::cananddevice::types::SettingFlags,
    ) -> Self {
        Self {
            index,
            value,
            flags,
        }
    }

    pub const fn with_flags(mut self, flags: crate::cananddevice::types::SettingFlags) -> Self {
        self.flags = flags;
        self
    }
}
impl WrapperSerializable for SetSetting {
    fn try_from_wrapper<T: crate::CanandMessage<T>>(
        cmsg: &crate::CanandMessageWrapper<T>,
    ) -> Result<Self, MessageCastError> {
        let id = cmsg.get_id();

        if !api_index_match(id, cananddevice::MessageIndex::SetSetting.into()) {
            return Err(MessageCastError::WrongMessage(((id >> 6) & 0xff) as u8));
        }

        let mut data = [0u8; 8];
        let len = cmsg.get_len() as usize;
        data[..len].copy_from_slice(&cmsg.get_data()[..len]);
        Ok(data.into())
    }

    fn try_into_wrapper<T: crate::CanandMessage<T>>(
        &self,
        device_type: u8,
        device_id: u8,
    ) -> Result<crate::CanandMessageWrapper<T>, CanandMessageError> {
        let data: [u8; 8] = (*self).into();
        Ok(crate::CanandMessageWrapper(T::try_from_data(
            build_frc_can_id(
                device_type,
                crate::REDUX_VENDOR_ID,
                cananddevice::MessageIndex::SetSetting as u16,
                device_id,
            ),
            &data,
        )?))
    }
}

impl From<[u8; 8]> for SetSetting {
    fn from(value: [u8; 8]) -> Self {
        Self {
            index: value[0],
            value: value[1..7].try_into().unwrap(),
            // TODO: this really needs to be type-encoded somehow. Hardcoding this is Bad long-term
            flags: cananddevice::types::SettingFlags {
                ephemeral: (value[7] & 0b1) != 0,
                synch_hold: (value[7] & 0b10) != 0,
                synch_msg_count: (value[7] >> 4),
            },
        }
    }
}

impl From<SetSetting> for [u8; 8] {
    fn from(value: SetSetting) -> Self {
        let d = value.value;
        let flags = value.flags.ephemeral as u8
            | (value.flags.synch_hold as u8) << 1
            | (value.flags.synch_msg_count << 4);

        [value.index, d[0], d[1], d[2], d[3], d[4], d[5], flags]
    }
}

macro_rules! impl_set_setting {
    ($dev:ident) => {
        impl TryFrom<crate::$dev::Message> for SetSetting {
            type Error = MessageCastError;

            fn try_from(value: crate::$dev::Message) -> Result<Self, Self::Error> {
                match value {
                    crate::$dev::Message::SetSetting {
                        address,
                        value,
                        flags,
                    } => Ok(Self {
                        index: address.into(),
                        value,
                        flags,
                    }),
                    _ => Err(MessageCastError::WrongMessage(value.raw_message_index())),
                }
            }
        }

        impl TryFrom<SetSetting> for crate::$dev::Message {
            type Error = SettingCastError;

            fn try_from(value: SetSetting) -> Result<Self, Self::Error> {
                Ok(crate::$dev::Message::SetSetting {
                    address: value
                        .index
                        .try_into()
                        .map_err(|_| SettingCastError::InvalidIndex(value.index))?,
                    value: value.value,
                    flags: value.flags,
                })
            }
        }

        impl From<crate::$dev::Setting> for SetSetting {
            fn from(value: crate::$dev::Setting) -> Self {
                Self {
                    index: value.raw_index(),
                    value: value.into(),
                    flags: crate::$dev::types::SettingFlags {
                        ephemeral: false,
                        synch_hold: false,
                        synch_msg_count: 0,
                    },
                }
            }
        }

        impl TryFrom<SetSetting> for crate::$dev::Setting {
            type Error = SettingCastError;

            fn try_from(value: SetSetting) -> Result<Self, Self::Error> {
                crate::$dev::Setting::from_address_data(
                    value
                        .index
                        .try_into()
                        .map_err(|_| SettingCastError::InvalidIndex(value.index))?,
                    &value.value,
                )
                .map_err(|_| SettingCastError::InvalidData)
            }
        }
    };
}

#[cfg(feature = "canandmag")]
impl_set_setting!(canandmag);

#[cfg(feature = "canandgyro")]
impl_set_setting!(canandgyro);

#[cfg(feature = "canandcolor")]
impl_set_setting!(canandcolor);
