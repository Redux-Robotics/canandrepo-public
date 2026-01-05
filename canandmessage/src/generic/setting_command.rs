use super::{api_index_match, build_frc_can_id, MessageCastError, WrapperSerializable};
use crate::{cananddevice, traits::CanandDeviceMessage, CanandMessageError};

#[cfg_attr(feature = "device", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SettingCommand {
    FetchSettings,
    ResetFactoryDefault,
    FetchSettingValue(u8),
    Other(u8),
}

impl WrapperSerializable for SettingCommand {
    fn try_from_wrapper<T: crate::CanandMessage<T>>(
        cmsg: &crate::CanandMessageWrapper<T>,
    ) -> Result<Self, MessageCastError> {
        let id = cmsg.get_id();
        let len = cmsg.get_len() as usize;
        if !api_index_match(id, cananddevice::MessageIndex::SettingCommand.into()) {
            return Err(MessageCastError::WrongMessage(((id >> 6) & 0xff) as u8));
        }

        if cmsg.get_len() < 1 {
            return Err(MessageCastError::WrongDlc(cmsg.get_len()));
        }

        let mut data: [u8; 8] = [0u8; 8];
        data[..len].copy_from_slice(cmsg.get_data()[..len].try_into().unwrap());

        Ok(
            match cananddevice::types::SettingCommand::try_from(data[0]) {
                Ok(i) => match i {
                    cananddevice::types::SettingCommand::FetchSettings => {
                        SettingCommand::FetchSettings
                    }
                    cananddevice::types::SettingCommand::ResetFactoryDefault => {
                        SettingCommand::ResetFactoryDefault
                    }
                    cananddevice::types::SettingCommand::FetchSettingValue => {
                        SettingCommand::FetchSettingValue(data[1])
                    }
                },
                Err(_) => SettingCommand::Other(data[0]),
            },
        )
    }

    fn try_into_wrapper<T: crate::CanandMessage<T>>(
        &self,
        device_type: u8,
        device_id: u8,
    ) -> Result<crate::CanandMessageWrapper<T>, CanandMessageError> {
        let arb_id = build_frc_can_id(
            device_type,
            crate::REDUX_VENDOR_ID,
            cananddevice::MessageIndex::SettingCommand as u16,
            device_id,
        );
        Ok(match self {
            SettingCommand::FetchSettings => crate::CanandMessageWrapper(T::try_from_data(
                arb_id,
                &[cananddevice::types::SettingCommand::FetchSettings as u8],
            )?),
            SettingCommand::ResetFactoryDefault => crate::CanandMessageWrapper(T::try_from_data(
                arb_id,
                &[cananddevice::types::SettingCommand::ResetFactoryDefault as u8],
            )?),
            SettingCommand::FetchSettingValue(idx) => {
                crate::CanandMessageWrapper(T::try_from_data(
                    arb_id,
                    &[
                        cananddevice::types::SettingCommand::FetchSettingValue as u8,
                        *idx,
                    ],
                )?)
            }
            SettingCommand::Other(idx) => {
                crate::CanandMessageWrapper(T::try_from_data(arb_id, &[*idx])?)
            }
        })
    }
}

macro_rules! impl_conv {
    ($dev:ident) => {
        impl TryFrom<crate::$dev::Message> for SettingCommand {
            type Error = MessageCastError;

            fn try_from(value: crate::$dev::Message) -> Result<Self, Self::Error> {
                match value {
                    crate::$dev::Message::SettingCommand {
                        control_flag,
                        setting_index,
                    } => Ok(match control_flag {
                        crate::$dev::types::SettingCommand::FetchSettings => {
                            SettingCommand::FetchSettings
                        }
                        crate::$dev::types::SettingCommand::ResetFactoryDefault => {
                            SettingCommand::ResetFactoryDefault
                        }
                        crate::$dev::types::SettingCommand::FetchSettingValue => {
                            match setting_index {
                                Some(idx) => SettingCommand::FetchSettingValue(idx.into()),
                                None => return Err(MessageCastError::InvalidMessage),
                            }
                        }
                        #[allow(unreachable_patterns)]
                        _ => SettingCommand::Other(control_flag as u8),
                    }),
                    _ => Err(MessageCastError::WrongMessage(value.raw_message_index())),
                }
            }
        }

        impl TryFrom<SettingCommand> for crate::$dev::Message {
            type Error = MessageCastError;

            fn try_from(value: SettingCommand) -> Result<Self, Self::Error> {
                Ok(match &value {
                    SettingCommand::FetchSettingValue(idx) => {
                        crate::$dev::Message::SettingCommand {
                            control_flag: crate::$dev::types::SettingCommand::FetchSettingValue,
                            setting_index: Some(
                                crate::$dev::types::Setting::try_from(*idx)
                                    .map_err(|_| MessageCastError::InvalidMessage)?,
                            ),
                        }
                    }
                    SettingCommand::FetchSettings => crate::$dev::Message::SettingCommand {
                        control_flag: crate::$dev::types::SettingCommand::FetchSettings,
                        setting_index: None,
                    },
                    SettingCommand::ResetFactoryDefault => crate::$dev::Message::SettingCommand {
                        control_flag: crate::$dev::types::SettingCommand::ResetFactoryDefault,
                        setting_index: None,
                    },
                    SettingCommand::Other(idx) => crate::$dev::Message::SettingCommand {
                        control_flag: crate::$dev::types::SettingCommand::try_from(*idx)
                            .map_err(|_| MessageCastError::InvalidMessage)?,
                        setting_index: None,
                    },
                })
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
