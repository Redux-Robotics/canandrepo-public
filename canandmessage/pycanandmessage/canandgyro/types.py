import enum
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *

class AtomicBondBusRate(enum.IntEnum):
    RATE_1M_2B = 0x0
    """1 megabit/s CAN 2.0B"""
    RATE_RESERVED_0 = 0x1
    """1 megabit/s CAN-FD"""
    RATE_RESERVED_1 = 0x2
    """5 megabit/s CAN-FD"""
    RATE_RESERVED_2 = 0x3
    """8 megabit/s CAN-FD"""

class CalibrationType(enum.IntEnum):
    NORMAL = 0x0
    """Normal calibration routine"""
    SAVE_ZRO = 0x1
    """Save ZRO at calibration complete"""
    TEMP_CAL_0 = 0x2
    """Temperature calibrate slot 0"""
    TEMP_CAL_1 = 0x3
    """Temperature calibrate slot 1"""

class Setting(enum.IntEnum):
    CAN_ID = 0x0
    """CAN Device ID"""
    NAME_0 = 0x1
    """device_name[0:5]"""
    NAME_1 = 0x2
    """device_name[6:11]"""
    NAME_2 = 0x3
    """device_name[12:17]"""
    STATUS_FRAME_PERIOD = 0x4
    """Status frame period (ms)"""
    SERIAL_NUMBER = 0x5
    """Serial number"""
    FIRMWARE_VERSION = 0x6
    """Firmware version"""
    CHICKEN_BITS = 0x7
    """Device-specific chicken bits"""
    DEVICE_TYPE = 0x8
    """Device-specific type identifier"""
    SCRATCH_0 = 0x9
    """User-writable scratch bytes 1"""
    SCRATCH_1 = 0xa
    """User-writable scratch bytes 2"""
    YAW_FRAME_PERIOD = 0xff
    """Yaw angle frame period (ms)"""
    ANGULAR_POSITION_FRAME_PERIOD = 0xfe
    """Angular position frame period (ms)"""
    ANGULAR_VELOCITY_FRAME_PERIOD = 0xfd
    """Angular velocity frame period (ms)"""
    ACCELERATION_FRAME_PERIOD = 0xfc
    """Acceleration frame period (ms)"""
    SET_YAW = 0xfb
    """Set yaw"""
    SET_POSE_POSITIVE_W = 0xfa
    """Set (normed) quaternion assuming positive W"""
    SET_POSE_NEGATIVE_W = 0xf9
    """Set (normed) quaternion assuming negative W"""
    GYRO_X_SENSITIVITY = 0xf8
    """Gyro X axis sensitivity"""
    GYRO_Y_SENSITIVITY = 0xf7
    """Gyro Y axis sensitivity"""
    GYRO_Z_SENSITIVITY = 0xf6
    """Gyro Z axis sensitivity"""
    GYRO_X_ZRO_OFFSET = 0xf5
    """Gyro X-axis calibrated ZRO offset"""
    GYRO_Y_ZRO_OFFSET = 0xf4
    """Gyro Y-axis calibrated ZRO offset"""
    GYRO_Z_ZRO_OFFSET = 0xf3
    """Gyro Z-axis calibrated ZRO offset"""
    GYRO_ZRO_OFFSET_TEMPERATURE = 0xf2
    """Temperature at ZRO offset (celsius)"""
    TEMPERATURE_CALIBRATION_X_0 = 0xe7
    """Temp cal X-axis point 0"""
    TEMPERATURE_CALIBRATION_Y_0 = 0xe6
    """Temp cal Y-axis point 0"""
    TEMPERATURE_CALIBRATION_Z_0 = 0xe5
    """Temp cal Z-axis point 0"""
    TEMPERATURE_CALIBRATION_T_0 = 0xe4
    """Temp cal temperature point 0 (celsius)"""
    TEMPERATURE_CALIBRATION_X_1 = 0xe3
    """Temp cal X-axis point 1"""
    TEMPERATURE_CALIBRATION_Y_1 = 0xe2
    """Temp cal Y-axis point 1"""
    TEMPERATURE_CALIBRATION_Z_1 = 0xe1
    """Temp cal Z-axis point 1"""
    TEMPERATURE_CALIBRATION_T_1 = 0xe0
    """Temp cal temperature point 1 (celsius)"""

class SettingCommand(enum.IntEnum):
    FETCH_SETTINGS = 0x0
    """Fetch all settings from device via a series of :ref:`report setting<msg_report_setting>` messages of all indexes"""
    RESET_FACTORY_DEFAULT = 0x1
    """Reset all resettanble settings to factory default, and broadcast all setting values via
    :ref:`report setting<msg_report_setting>` messages.
    """
    FETCH_SETTING_VALUE = 0x2
    """Requests to fetch a single setting from device, with its value reported via the 
    :ref:`report setting<msg_report_setting>` message. 

    This requires the use of the second byte to specify the setting index to fetch."""

class SettingReportFlags(enum.Flag):
    SET_SUCCESS = 0x1
    """Whether the setting set/fetch was successful"""
    COMMIT_SUCCESS = 0x2
    """Whether the setting synch commit was successful"""

class AtomicAnnouncementFlags(enum.Flag):
    NEGOTIATION = 0x1
    """Device should enter negotiation phase"""
    INIT = 0x2
    """Device should initialize bus with new rate"""
    CONFIRM = 0x4
    """Device should confirm new bus rate"""
    BEGIN_TX = 0x8
    """Device should begin transmission"""
    BUS_INTERRUPT = 0x10
    """Device should cease all transmission"""

class Faults(enum.Flag):
    POWER_CYCLE = 0x1
    """The power cycle fault flag, which is set to true when the device first boots.
    Clearing sticky faults and then checking this flag can be used to determine if the device rebooted.
    """
    CAN_ID_CONFLICT = 0x2
    """The CAN ID conflict flag, which is set to true if there is a CAN id conflict.
    In practice, you should physically inspect the device to ensure it's not flashing blue.
    """
    CAN_GENERAL_ERROR = 0x4
    """The CAN general error flag, which will raise if the device encounters a CAN fault during operation.
    If communication with the device still functions, this will not register as an active fault for long if at all.
    This may raise due to wiring issues, such as an intermittently shorted CAN bus.
    """
    OUT_OF_TEMPERATURE_RANGE = 0x8
    """The temperature range flag, which will raise if the device is not between 0-70 degrees Celsius.
    This may be of concern if the device is near very active motors.
    """
    HARDWARE_FAULT = 0x10
    """The hardware fault flag, which will raise if a hardware issue is detected.
    Generally will raise if the device's controller cannot read the physical sensor itself.
    """
    CALIBRATING = 0x20
    """The calibration status flag, which will raise if the device is currently calibrating.
    """
    ANGULAR_VELOCITY_SATURATION = 0x40
    """The angular velocity saturation flag, which triggers on saturation of angular velocity.
    """
    ACCELERATION_SATURATION = 0x80
    """The acceleration saturation flag, which triggers on saturation of acceleration.
    """


@dataclasses.dataclass
class SettingFlags:
    ephemeral: Annotated[bool, Signal(0, Boolean(False))]
    """Whether the setting should be set ephemeral"""
    synch_hold: Annotated[bool, Signal(1, Boolean(False))]
    """Whether the setting should be held until the next synch barrier"""
    synch_msg_count: Annotated[int, Signal(4, UInt(width=4, min=0, max=15, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Synch message count"""


@dataclasses.dataclass
class FirmwareVersion:
    firmware_patch: Annotated[int, Signal(0, UInt(width=8, min=0, max=255, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Firmware version patch number"""
    firmware_minor: Annotated[int, Signal(8, UInt(width=8, min=0, max=255, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Firmware version minor number"""
    firmware_year: Annotated[int, Signal(16, UInt(width=16, min=0, max=65535, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Firmware version year"""


@dataclasses.dataclass
class TempCalPoint:
    temperature_point: Annotated[int, Signal(0, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=1, factor_den=256, offset=0))]
    """Temperature point"""
    offset: Annotated[float, Signal(16, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Offset at the temperature"""


@dataclasses.dataclass
class QuatXyz:
    x: Annotated[int, Signal(0, SInt(width=16, min=-32767, max=32767, default_value=0, factor_num=1, factor_den=32767, offset=0))]
    """Quaternion x term"""
    y: Annotated[int, Signal(16, SInt(width=16, min=-32767, max=32767, default_value=0, factor_num=1, factor_den=32767, offset=0))]
    """Quaternion y term"""
    z: Annotated[int, Signal(32, SInt(width=16, min=-32767, max=32767, default_value=0, factor_num=1, factor_den=32767, offset=0))]
    """Quaternion z term"""


@dataclasses.dataclass
class Yaw:
    yaw: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0, allow_nan_inf=True, factor_num=1, factor_den=1, offset=0))]
    """Yaw angle (f32 between [-pi..pi) radians)"""
    wraparound: Annotated[int, Signal(32, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Wraparound counter"""

