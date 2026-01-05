//! FRC CAN ID helpers.
//!
//! For more information, see
//! [the official documentation.](https://docs.wpilib.org/en/stable/docs/software/can-devices/can-addressing.html)
#![no_std]
#![warn(missing_docs)]
use num_enum::{FromPrimitive, IntoPrimitive};
/// ID of the CAN heartbeat.
pub const HEARTBEAT_ID: u32 = 0x01011840;
/// Redux vendor id.
pub const REDUX_VENDOR_ID: u8 = 0xe;
/// Redux enumerate broadcast id.
pub const REDUX_BROADCAST_ENUMERATE: u32 = build_frc_can_id(0, 0xe, 0, 0);
/// Generic filter for a device id.
pub const DEVICE_FILTER: u32 = build_frc_can_id(0x1f, 0xff, 0, 0x3f);
/// Global disable actuators packet id.
pub const GLOBAL_DISABLE: u32 = 0;

/// Newtype for an FRC CAN ID.
pub struct FRCCanId(pub u32);
impl FRCCanId {
    /// Build an ID from constituent parts.
    ///
    /// No checks are done to ensure that `api_index <= 1023` and `device_number <= 63`.
    pub const fn build(
        device_type: FRCCanDeviceType,
        mfg: FRCCanVendor,
        api_idx: u16,
        device_number: u8,
    ) -> Self {
        Self(build_frc_can_id(
            device_type.as_u8(),
            mfg.as_u8(),
            api_idx,
            device_number,
        ))
    }

    /// Instantiates a new id from a raw 29-bit id.
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Gets the device number.
    pub const fn device_number(&self) -> u8 {
        (self.0 & 0x3f) as u8
    }

    /// Gets the API index.
    pub const fn api_index(&self) -> u16 {
        ((self.0 >> 6) & 0x3ff) as u16
    }

    /// Gets the raw manufacturer code.
    pub const fn manufacturer_code(&self) -> u8 {
        ((self.0 >> 16) & 0xff) as u8
    }

    /// Gets the manufacturer as an enum.
    pub fn manufacturer(&self) -> FRCCanVendor {
        FRCCanVendor::from(self.manufacturer_code())
    }

    /// Gets the device type id.
    pub const fn device_type_code(&self) -> u8 {
        ((self.0 >> 24) & 0x1f) as u8
    }

    /// Gets the device type as an enum.
    pub fn device_type(&self) -> FRCCanDeviceType {
        FRCCanDeviceType::from(self.device_type_code())
    }
}

impl From<u32> for FRCCanId {
    fn from(value: u32) -> Self {
        Self(value)
    }
}

/// Newtype for a roboRIO CAN heartbeat.
///
/// If this packet is not seen for ~100 milliseconds, or [`Self::system_watchdog`] returns false,
/// then the robot's actuators are expected to be disabled.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FRCCanHeartbeat(u64);

impl FRCCanHeartbeat {
    /// Instantiates from an array.
    pub const fn new(data: [u8; 8]) -> Self {
        Self(u64::from_be_bytes(data))
    }

    /// Raw daya bytes.
    pub const fn data(&self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    /// Match time in seconds
    pub const fn match_time_seconds(&self) -> u8 {
        // u8
        self.0 as u8
    }

    /// Match number
    pub const fn match_number(&self) -> u16 {
        // u10
        ((self.0 >> 8) & 0x3ff) as u16
    }

    /// Replay number
    pub const fn replay_number(&self) -> u8 {
        // u6
        ((self.0 >> 18) & 0x3f) as u8
    }

    /// True if on the red alliance
    pub const fn red_alliance(&self) -> bool {
        (self.0 & (1 << 24)) != 0
    }

    /// True if the robot is enabled.
    pub const fn enabled(&self) -> bool {
        (self.0 & (1 << 25)) != 0
    }

    /// True if it is currently autonomous.
    pub const fn autonomous(&self) -> bool {
        (self.0 & (1 << 26)) != 0
    }

    /// True if the DS indicates test mode.
    pub const fn test_mode(&self) -> bool {
        (self.0 & (1 << 27)) != 0
    }

    /// True if motors can be energized.
    ///
    /// THIS IS THE ONLY FLAG THAT MATTERS FOR MOTOR SAFETY.
    pub const fn system_watchdog(&self) -> bool {
        (self.0 & (1 << 28)) != 0
    }

    /// Tournament type
    pub const fn tournament_type(&self) -> u8 {
        // u3
        ((self.0 >> 29) & 0b111) as u8
    }

    /// Time of day (year)
    pub const fn time_of_day_year(&self) -> u8 {
        // u6
        ((self.0 >> 32) & 0x3f) as u8
    }

    /// Time of day (month)
    pub const fn time_of_day_month(&self) -> u8 {
        // u4
        ((self.0 >> 38) & 0xf) as u8
    }

    /// Time of day (day)
    pub const fn time_of_day_day(&self) -> u8 {
        // u5
        ((self.0 >> 42) & 0x1f) as u8
    }

    /// Time of day (seconds)
    pub const fn time_of_day_sec(&self) -> u8 {
        // u6
        ((self.0 >> 47) & 0x3f) as u8
    }

    /// Time of day (minutes)
    pub const fn time_of_day_min(&self) -> u8 {
        // u6
        ((self.0 >> 53) & 0x3f) as u8
    }

    /// Time of day (hours)
    pub const fn time_of_day_hour(&self) -> u8 {
        // u5
        ((self.0 >> 59) & 0x1f) as u8
    }
}

impl core::fmt::Debug for FRCCanHeartbeat {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RoboRioHeartbeat")
            .field("data", &self.data())
            .field("match_time_seconds", &self.match_time_seconds())
            .field("match_number", &self.match_number())
            .field("replay_number", &self.replay_number())
            .field("red_alliance", &self.red_alliance())
            .field("enabled", &self.enabled())
            .field("autonomous", &self.autonomous())
            .field("test_mode", &self.test_mode())
            .field("system_watchdog", &self.system_watchdog())
            .field("tournament_type", &self.tournament_type())
            .field("time_of_day_year", &self.time_of_day_year())
            .field("time_of_day_month", &self.time_of_day_month())
            .field("time_of_day_day", &self.time_of_day_day())
            .field("time_of_day_sec", &self.time_of_day_sec())
            .field("time_of_day_min", &self.time_of_day_min())
            .field("time_of_day_hour", &self.time_of_day_hour())
            .finish()
    }
}

/// Device type (most significant 5 bits )
#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, IntoPrimitive, Hash,
)]
#[repr(u8)]
pub enum FRCCanDeviceType {
    /// Broadcast type; not device specific
    Broadcast = 0,
    /// Robot controller
    RobotController = 1,
    /// Motor controller
    MotorController = 2,
    /// Relay controller
    RelayController = 3,
    /// Gyro sensor
    GyroSensor = 4,
    /// Accelerometer
    Accelerometer = 5,
    /// Distance sensor (formerly "ultrasonic sensor")
    DistanceSensor = 6,
    /// Encoder (formerly "gear tooth sensor")
    Encoder = 7,
    /// Power distribution module (e.g. PDP/H)
    PowerDistributionModule = 8,
    /// Pneumatics controller
    PneumaticsController = 9,
    /// Misc.
    Miscellaneous = 10,
    /// IO breakout
    IOBreakout = 11,
    /// Firmware update
    FirmwareUpdate = 31,
    /// Everything else
    #[num_enum(catch_all)]
    Reserved(u8),
}

// PLEASE give us impl_trait jesus christ
// Also see: https://github.com/illicitonion/num_enum/pull/147
impl FRCCanDeviceType {
    /// Const conversion to a u8
    pub const fn as_u8(&self) -> u8 {
        match self {
            Self::Broadcast => 0,
            Self::RobotController => 1,
            Self::MotorController => 2,
            Self::RelayController => 3,
            Self::GyroSensor => 4,
            Self::Accelerometer => 5,
            Self::DistanceSensor => 6,
            Self::Encoder => 7,
            Self::PowerDistributionModule => 8,
            Self::PneumaticsController => 9,
            Self::Miscellaneous => 10,
            Self::IOBreakout => 11,
            Self::FirmwareUpdate => 31,
            Self::Reserved(x) => *x,
        }
    }
}

/// Non-exhaustive list of FRC CAN vendors.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
#[non_exhaustive]
pub enum FRCCanVendor {
    /// Global broadcast (e.g. for global disable)
    Broadcast = 0,
    /// National Instruments
    NationalInstruments = 1,
    /// Luminary Micro (the original manufacturer of the Jaguar)
    LuminaryMicro = 2,
    /// DEKA
    Deka = 3,
    /// CTR-Electronics
    CtrElectronics = 4,
    /// Rev Robotics
    Rev = 5,
    /// Grapple
    Grapple = 6,
    /// MindSensors
    MindSensors = 7,
    /// Reserved for team use custom electronics
    TeamUse = 8,
    /// Kauai Labs
    KauaiLabs = 9,
    /// //Cu
    Copperforge = 10,
    /// Playing with Fusion
    PlayingWithFusion = 11,
    /// Studica
    Studica = 12,
    /// The Thrifty Bot
    ThriftyBot = 13,
    /// Redux Robotics
    Redux = 14,
    /// AndyMark
    AndyMark = 15,
    /// Vivid Hosting
    VividHosting = 16,
    /// Vertos Robotics
    Vertos = 17,
    /// SWYFT Robotics
    Swyft = 18,
    /// Lumyn Labs
    LumynLabs = 19,
    /// Brushland Labs
    BrushlandLabs = 20,
    /// Other vendor
    #[num_enum(catch_all)]
    Unknown(u8),
}

impl FRCCanVendor {
    /// Const conversion to u8
    pub const fn as_u8(&self) -> u8 {
        match self {
            Self::Broadcast => 0,
            Self::NationalInstruments => 1,
            Self::LuminaryMicro => 2,
            Self::Deka => 3,
            Self::CtrElectronics => 4,
            Self::Rev => 5,
            Self::Grapple => 6,
            Self::MindSensors => 7,
            Self::TeamUse => 8,
            Self::KauaiLabs => 9,
            Self::Copperforge => 10,
            Self::PlayingWithFusion => 11,
            Self::Studica => 12,
            Self::ThriftyBot => 13,
            Self::Redux => 14,
            Self::AndyMark => 15,
            Self::VividHosting => 16,
            Self::Vertos => 17,
            Self::Swyft => 18,
            Self::LumynLabs => 19,
            Self::BrushlandLabs => 20,
            Self::Unknown(x) => *x,
        }
    }
}

/// Raw FRC CAN ID builder
pub const fn build_frc_can_id(
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

#[test]
fn test_roborio_hb() {
    let hb_raw_disabled = [0xb8, 0x4e, 0x0e, 0xbc, 0x00, 0x00, 0x00, 0xff];
    let hb = FRCCanHeartbeat::new(hb_raw_disabled);
    assert!(!hb.system_watchdog());
    let hb_raw_enabled = [0x39, 0xc7, 0x0e, 0x7d, 0x13, 0x00, 0x00, 0xff];
    let hb = FRCCanHeartbeat::new(hb_raw_enabled);
    assert!(hb.system_watchdog());
    let hb_raw_no_watchdog = [0x39, 0xd7, 0x0e, 0x7d, 0x02, 0x00, 0x00, 0xff];
    let hb = FRCCanHeartbeat::new(hb_raw_no_watchdog);
    assert!(!hb.system_watchdog());
}
