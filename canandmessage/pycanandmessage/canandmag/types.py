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
    ZERO_OFFSET = 0xff
    """Encoder zero offset"""
    VELOCITY_WINDOW = 0xfe
    """Velocity window width (value*250us)"""
    POSITION_FRAME_PERIOD = 0xfd
    """Position frame period (ms)"""
    VELOCITY_FRAME_PERIOD = 0xfc
    """Velocity frame period (ms)"""
    RAW_POSITION_FRAME_PERIOD = 0xfb
    """Raw position frame period (ms)"""
    INVERT_DIRECTION = 0xfa
    """Invert direction (0=ccw, 1=cw)"""
    RELATIVE_POSITION = 0xf9
    """Set relative position value"""
    DISABLE_ZERO_BUTTON = 0xf8
    """Disable the zero button"""

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
    RESET_FACTORY_DEFAULT_KEEP_ZERO = 0xff
    """Reset to factory defaults, but keep encoder zero offset"""

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
    """The power cycle fault flag, which is set to true when the encoder first boots.
    Clearing sticky faults and then checking this flag can be used to determine if the encoder rebooted.
    """
    CAN_ID_CONFLICT = 0x2
    """The CAN ID conflict flag, which is set to true if there is a CAN id conflict.
    In practice, you should physically inspect the encoder to ensure it's not flashing blue.
    """
    CAN_GENERAL_ERROR = 0x4
    """The CAN general error flag, which will raise if the device encounters a CAN fault during operation.
    If communication with the device still functions, this will not register as an active fault for long if at all.
    This may raise due to wiring issues, such as an intermittently shorted CAN bus.
    """
    OUT_OF_TEMPERATURE_RANGE = 0x8
    """The temperature range flag, which will raise if the encoder is not between 0-70 degrees Celsius.
    This may be of concern if the encoder is near very active motors.
    """
    HARDWARE_FAULT = 0x10
    """The hardware fault flag, which will raise if a hardware issue is detected.
    Generally will raise if the device's controller cannot read the physical sensor itself.
    """
    MAGNET_OUT_OF_RANGE = 0x20
    """The magnet out of range flag, which will raise if the measured shaft's magnet is not detected.
    This will match the encoder's LED shining red in normal operation.
    """
    UNDER_VOLT = 0x40
    """The undervolt flag, which will raise if the encoder is experiencing brownout conditions.
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
class ZeroOffset:
    offset_or_position: Annotated[int, Signal(0, UInt(width=14, min=0, max=16383, default_value=0, factor_num=1, factor_den=16384, offset=0))]
    """Zero offset or position"""
    position_bit: Annotated[bool, Signal(16, Boolean(False))]
    """True to set position instead of a zero offset."""

