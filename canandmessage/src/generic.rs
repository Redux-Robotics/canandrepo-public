pub(crate) const fn build_frc_can_id(
    device_type: u8,
    mfg_code: u8,
    api_idx: u16,
    device_number: u8,
) -> u32 {
    ((device_type as u32) << 24)
        | ((mfg_code as u32) << 16)
        | ((api_idx as u32) << 6)
        | device_number as u32
}

pub(crate) const fn api_index_match(id: u32, index: u8) -> bool {
    id & build_frc_can_id(0, 0xff, 0x3ff, 0)
        == build_frc_can_id(0, crate::REDUX_VENDOR_ID, index as u16, 0)
}

pub struct CanMaskFilter {
    pub expect: u32,
    pub mask: u32,
}

#[cfg_attr(feature = "device", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SettingCastError {
    InvalidIndex(u8),
    InvalidData,
}

#[cfg_attr(feature = "device", derive(defmt::Format))]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum MessageCastError {
    WrongMessage(u8),
    WrongDlc(u8),
    InvalidMessage,
}

pub trait WrapperSerializable: Sized {
    fn try_from_wrapper<T: crate::CanandMessage<T>>(
        cmsg: &crate::CanandMessageWrapper<T>,
    ) -> Result<Self, MessageCastError>;
    fn try_into_wrapper<T: crate::CanandMessage<T>>(
        &self,
        device_type: u8,
        device_id: u8,
    ) -> Result<crate::CanandMessageWrapper<T>, CanandMessageError>;
}

mod report_setting;
pub use report_setting::*;

mod set_setting;
pub use set_setting::*;

mod buf_msg;
pub use buf_msg::*;

mod enumerate;
pub use enumerate::*;

mod setting_command;
pub use setting_command::*;

use crate::CanandMessageError;
