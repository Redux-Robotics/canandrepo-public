use core::fmt::Debug;

use crate::CanandMessageError;

pub trait CanandDevice: Debug + PartialEq + Eq + Clone + Copy {
    type Message: CanandDeviceMessage;
    type Setting: CanandDeviceSetting;

    const DEV_TYPE: u8;
    const DEV_NAME: &'static str;

    fn setting_info<'a>() -> &'a [SettingInfo<Self::Setting>];
}

/// Device messages.
pub trait CanandDeviceMessage: Sized + core::fmt::Debug {
    #[cfg(feature = "device")]
    type Index: TryFrom<u8, Error = ()>
        + Into<u8>
        + Debug
        + PartialEq
        + Eq
        + Clone
        + Copy
        + PartialOrd
        + Ord
        + defmt::Format;
    #[cfg(not(feature = "device"))]
    type Index: TryFrom<u8, Error = ()>
        + Into<u8>
        + Debug
        + PartialEq
        + Eq
        + Clone
        + Copy
        + PartialOrd
        + Ord;

    /// With the use of a device id, converts the current message into a CanandMessageWrapper which can be dereferenced into type T.
    fn try_into_wrapper<T: crate::CanandMessage<T>>(
        &self,
        can_device_id: u32,
    ) -> Result<crate::CanandMessageWrapper<T>, CanandMessageError>;

    /// Gets the message index as a u16.
    fn raw_message_index(&self) -> u8 {
        // unsafe { *<*const _>::from(self).cast::<u8>() }
        unsafe { *(self as *const Self).cast::<u8>() }
    }

    /// Gets the message index as the associated MessageIndex enum.
    fn message_index(&self) -> Self::Index {
        Self::Index::try_from(self.raw_message_index()).unwrap()
    }

    /// Calls TryFrom::try_from(Self) to convert from a transport message to an internal representation.
    ///
    /// Limitations in type systems require this to be explicit.
    fn try_from_wrapper<T: crate::CanandMessage<T>>(
        cmsg: &crate::CanandMessageWrapper<T>,
    ) -> Result<Self, ()>;
}

pub trait CanandDeviceSetting: Into<[u8; 6]> + Debug + PartialEq + Clone + Copy {
    #[cfg(feature = "device")]
    type Index: TryFrom<u8, Error = ()>
        + Into<u8>
        + Debug
        + PartialEq
        + Eq
        + Clone
        + Copy
        + PartialOrd
        + Ord
        + defmt::Format;
    #[cfg(not(feature = "device"))]
    type Index: TryFrom<u8, Error = ()>
        + Into<u8>
        + Debug
        + PartialEq
        + Eq
        + Clone
        + Copy
        + PartialOrd
        + Ord;

    /// Converts an address/data pair into an instance.
    fn from_address_data(address: Self::Index, data: &[u8; 6]) -> Result<Self, ()>;

    /// Gets the message index as a u8.
    fn raw_index(&self) -> u8 {
        // unsafe { *<*const _>::from(self).cast::<u8>() }
        unsafe { *(self as *const Self).cast::<u8>() }
    }

    /// Gets the message index as the associated MessageIndex enum.
    fn setting_index(&self) -> Self::Index {
        Self::Index::try_from(self.raw_index()).unwrap()
    }
}

pub struct SettingInfo<S: CanandDeviceSetting> {
    pub readable: bool,
    pub writable: bool,
    pub reset_on_default: bool,
    pub index: S::Index,
    pub default_value: S,
}

pub trait Bitset<U> {
    fn set_index(&mut self, idx: u32, value: bool);
    fn get_index(&self, idx: u32) -> bool;
    fn value(&self) -> U;
}
