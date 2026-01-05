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

