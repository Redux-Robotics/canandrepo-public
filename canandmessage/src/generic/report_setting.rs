use super::{
    api_index_match, build_frc_can_id, MessageCastError, SettingCastError, WrapperSerializable,
};
use crate::{
    cananddevice,
    traits::{Bitset, CanandDeviceMessage, CanandDeviceSetting},
};

#[cfg_attr(feature = "device", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ReportSetting {
    pub index: u8,
    pub value: [u8; 6],
    pub flags: crate::cananddevice::types::SettingReportFlags,
}

impl ReportSetting {
    pub const fn new(
        index: u8,
        value: [u8; 6],
        flags: crate::cananddevice::types::SettingReportFlags,
    ) -> Self {
        Self {
            index,
            value,
            flags,
        }
    }

    pub const fn with_flags(
        mut self,
        flags: crate::cananddevice::types::SettingReportFlags,
    ) -> Self {
        self.flags = flags;
        self
    }

    //pub fn try_into_device_msg<M: CanandDeviceMessage + TryFrom<ReportSetting>>(&self) -> Result<M, ()> {
    //    M::try_from(*self)
    //}
}

impl WrapperSerializable for ReportSetting {
    fn try_from_wrapper<T: crate::CanandMessage<T>>(
        cmsg: &crate::CanandMessageWrapper<T>,
    ) -> Result<Self, MessageCastError> {
        let id = cmsg.get_id();
        if !api_index_match(id, cananddevice::MessageIndex::ReportSetting.into()) {
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
    ) -> Result<crate::CanandMessageWrapper<T>, crate::CanandMessageError> {
        let data: [u8; 8] = (*self).into();
        Ok(crate::CanandMessageWrapper(T::try_from_data(
            build_frc_can_id(
                device_type,
                crate::REDUX_VENDOR_ID,
                cananddevice::MessageIndex::ReportSetting as u16,
                device_id,
            ),
            &data,
        )?))
    }
}

impl From<[u8; 8]> for ReportSetting {
    fn from(value: [u8; 8]) -> Self {
        Self {
            index: value[0],
            value: value[1..7].try_into().unwrap(),
            flags: cananddevice::types::SettingReportFlags::from_bitfield(value[7]),
        }
    }
}

impl From<ReportSetting> for [u8; 8] {
    fn from(value: ReportSetting) -> Self {
        let d = value.value;
        [
            value.index,
            d[0],
            d[1],
            d[2],
            d[3],
            d[4],
            d[5],
            value.flags.value(),
        ]
    }
}

macro_rules! impl_report_setting {
    ($dev:ident) => {
        impl TryFrom<crate::$dev::Message> for ReportSetting {
            type Error = MessageCastError;

            fn try_from(value: crate::$dev::Message) -> Result<Self, Self::Error> {
                match value {
                    crate::$dev::Message::ReportSetting {
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

        impl TryFrom<ReportSetting> for crate::$dev::Message {
            type Error = SettingCastError;

            fn try_from(value: ReportSetting) -> Result<Self, Self::Error> {
                Ok(crate::$dev::Message::ReportSetting {
                    address: value
                        .index
                        .try_into()
                        .map_err(|_| SettingCastError::InvalidIndex(value.index))?,
                    value: value.value,
                    flags: value.flags,
                })
            }
        }

        impl From<crate::$dev::Setting> for ReportSetting {
            fn from(value: crate::$dev::Setting) -> Self {
                Self {
                    index: value.raw_index(),
                    value: value.into(),
                    flags: crate::$dev::types::SettingReportFlags::from_bitfield(0),
                }
            }
        }

        impl TryFrom<ReportSetting> for crate::$dev::Setting {
            type Error = SettingCastError;

            fn try_from(value: ReportSetting) -> Result<Self, Self::Error> {
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

impl_report_setting!(cananddevice);
#[cfg(feature = "canandmag")]
impl_report_setting!(canandmag);

#[cfg(feature = "canandgyro")]
impl_report_setting!(canandgyro);

#[cfg(feature = "canandcolor")]
impl_report_setting!(canandcolor);
