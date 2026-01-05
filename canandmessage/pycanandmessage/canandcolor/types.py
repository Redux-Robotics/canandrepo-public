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

class ExtraFrameMode(enum.IntEnum):
    DISABLED = 0x0
    """Do not emit extra frames beyond those specified in the frame period"""
    EARLY_TRANSMIT_ON_CHANGE = 0x1
    """Transmits a frame immidiately once readings change"""

class DigoutOutputConfig(enum.IntEnum):
    DISABLED = 0x0
    """Disable output on this pin"""
    DIGOUT_LOGIC_ACTIVE_HIGH = 0x1
    """Use digital logic pin, output 3.3v on true and 0v on false"""
    DIGOUT_LOGIC_ACTIVE_LOW = 0x2
    """Use digital logic pin, output 0v on true and 3.3v on false"""
    DUTY_CYCLE_OUTPUT = 0x3
    """Output a duty cycle on this pin. Only works on DIG-2"""

class SlotOpcode(enum.IntEnum):
    EQUALS = 0x0
    """true if ``a = b * (imm_scaling + 1) / 256 + (imm_additive)``"""
    LESS_THAN = 0x1
    """true if ``a < b * (imm_scaling + 1) / 256 + (imm_additive)``"""
    GREATER_THAN = 0x2
    """true if ``a > b * (imm_scaling + 1) / 256 + (imm_additive)``"""
    LESS_THAN_OR_EQUALS = 0x3
    """true if ``a <= b * (imm_scaling + 1) / 256 + (imm_additive)``"""
    GREATER_THAN_OR_EQUALS = 0x4
    """true if``a >= b * (imm_scaling + 1) / 256 + (imm_additive)``"""
    PREV_SLOT_TRUE = 0x5
    """true if previous slot true for ``b * (imm_scaling + 1) / 256 + (imm_additive)`` milliseconds"""
    PREV_CLAUSE_TRUE = 0x6
    """true if previous joined-slot-chain true for ``b * (imm_scaling + 1) / 256 + (imm_additive)`` milliseconds"""

class NextSlotAction(enum.IntEnum):
    TERMINATE_CHAIN = 0x0
    """Do not interact with the next slot"""
    OR_WITH_NEXT_SLOT = 0x1
    """Logical OR with next slot"""
    XOR_WITH_NEXT_SLOT = 0x2
    """Logical XOR with next slot"""
    AND_WITH_NEXT_SLOT = 0x3
    """Logical AND with next slot"""

class DataSource(enum.IntEnum):
    ZERO = 0x0
    """Always reads zero; can be used to compare only to additive immidate0"""
    DISTANCE = 0x1
    """Distance reading"""
    RED = 0x2
    """Red reading"""
    GREEN = 0x3
    """Green reading"""
    BLUE = 0x4
    """Blue reading"""
    HUE = 0x5
    """Hue reading"""
    SATURATION = 0x6
    """Saturation reading"""
    VALUE = 0x7
    """Value reading"""

class ColorIntegrationPeriod(enum.IntEnum):
    PERIOD_400_ms_RESOLUTION_20_bit = 0x0
    """400 ms - 20 bit resolution"""
    PERIOD_200_ms_RESOLUTION_19_bit = 0x1
    """200 ms - 19 bit resolution"""
    PERIOD_100_ms_RESOLUTION_18_bit = 0x2
    """100 ms - 18 bit resolution"""
    PERIOD_50_ms_RESOLUTION_17_bit = 0x3
    """50 ms - 17 bit resolution"""
    PERIOD_25_ms_RESOLUTION_16_bit = 0x4
    """25 ms - 16 bit resolution"""

class DistanceIntegrationPeriod(enum.IntEnum):
    PERIOD_5_ms = 0x0
    """5 millisecond period"""
    PERIOD_7p5_ms = 0x1
    """7.5 millisecond period"""
    PERIOD_10_ms = 0x2
    """10 millisecond period"""
    PERIOD_12p5_ms = 0x3
    """12.5 millisecond period"""
    PERIOD_15_ms = 0x4
    """15 millisecond period"""
    PERIOD_17p5_ms = 0x5
    """17.5 millisecond period"""
    PERIOD_20_ms = 0x6
    """20 millisecond period"""
    PERIOD_40_ms = 0x7
    """40 millisecond period"""

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
    DISTANCE_FRAME_PERIOD = 0xff
    """Distance frame period (ms)"""
    COLOR_FRAME_PERIOD = 0xfe
    """Color frame period (ms)"""
    DIGOUT_FRAME_PERIOD = 0xfd
    """Digout frame period (ms)"""
    DISTANCE_EXTRA_FRAME_MODE = 0xf7
    """Distance extra frame mode"""
    COLOR_EXTRA_FRAME_MODE = 0xf6
    """Color extra frame frame mode"""
    LAMP_BRIGHTNESS = 0xef
    """Lamp LED brightness"""
    COLOR_INTEGRATION_PERIOD = 0xee
    """Color integration period"""
    DISTANCE_INTEGRATION_PERIOD = 0xed
    """Distance integration period"""
    DIGOUT1_OUTPUT_CONFIG = 0xeb
    """Digital output 1 control config"""
    DIGOUT2_OUTPUT_CONFIG = 0xea
    """Digital output 2 control config"""
    DIGOUT1_MESSAGE_ON_CHANGE = 0xe9
    """Digital output 1 send message on change"""
    DIGOUT2_MESSAGE_ON_CHANGE = 0xe8
    """Digital output 2 send message on change"""
    DIGOUT1_CONFIG_0 = 0xd0
    """Digout1 config slot 0"""
    DIGOUT1_CONFIG_1 = 0xcf
    """Digout1 config slot 1"""
    DIGOUT1_CONFIG_2 = 0xce
    """Digout1 config slot 2"""
    DIGOUT1_CONFIG_3 = 0xcd
    """Digout1 config slot 3"""
    DIGOUT1_CONFIG_4 = 0xcc
    """Digout1 config slot 4"""
    DIGOUT1_CONFIG_5 = 0xcb
    """Digout1 config slot 5"""
    DIGOUT1_CONFIG_6 = 0xca
    """Digout1 config slot 6"""
    DIGOUT1_CONFIG_7 = 0xc9
    """Digout1 config slot 7"""
    DIGOUT1_CONFIG_8 = 0xc8
    """Digout1 config slot 8"""
    DIGOUT1_CONFIG_9 = 0xc7
    """Digout1 config slot 9"""
    DIGOUT1_CONFIG_10 = 0xc6
    """Digout1 config slot 10"""
    DIGOUT1_CONFIG_11 = 0xc5
    """Digout1 config slot 11"""
    DIGOUT1_CONFIG_12 = 0xc4
    """Digout1 config slot 12"""
    DIGOUT1_CONFIG_13 = 0xc3
    """Digout1 config slot 13"""
    DIGOUT1_CONFIG_14 = 0xc2
    """Digout1 config slot 14"""
    DIGOUT1_CONFIG_15 = 0xc1
    """Digout1 config slot 15"""
    DIGOUT2_CONFIG_0 = 0xc0
    """Digout2 config slot 0"""
    DIGOUT2_CONFIG_1 = 0xbf
    """Digout2 config slot 1"""
    DIGOUT2_CONFIG_2 = 0xbe
    """Digout2 config slot 2"""
    DIGOUT2_CONFIG_3 = 0xbd
    """Digout2 config slot 3"""
    DIGOUT2_CONFIG_4 = 0xbc
    """Digout2 config slot 4"""
    DIGOUT2_CONFIG_5 = 0xbb
    """Digout2 config slot 5"""
    DIGOUT2_CONFIG_6 = 0xba
    """Digout2 config slot 6"""
    DIGOUT2_CONFIG_7 = 0xb9
    """Digout2 config slot 7"""
    DIGOUT2_CONFIG_8 = 0xb8
    """Digout2 config slot 8"""
    DIGOUT2_CONFIG_9 = 0xb7
    """Digout2 config slot 9"""
    DIGOUT2_CONFIG_10 = 0xb6
    """Digout2 config slot 10"""
    DIGOUT2_CONFIG_11 = 0xb5
    """Digout2 config slot 11"""
    DIGOUT2_CONFIG_12 = 0xb4
    """Digout2 config slot 12"""
    DIGOUT2_CONFIG_13 = 0xb3
    """Digout2 config slot 13"""
    DIGOUT2_CONFIG_14 = 0xb2
    """Digout2 config slot 14"""
    DIGOUT2_CONFIG_15 = 0xb1
    """Digout2 config slot 15"""

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
    CLEAR_DIGOUT1 = 0xff
    """Clear all digout1 slots"""
    CLEAR_DIGOUT2 = 0xfe
    """Clear all digout2 slots"""
    FETCH_DIGOUT1 = 0xfd
    """Fetch all digout1 slots and settings"""
    FETCH_DIGOUT2 = 0xfc
    """Fetch all digout2 slots and settings"""

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
    HARDWARE_FAULT_DISTANCE = 0x10
    """The hardware fault flag corresponding to the distance sensor IC, which will raise if a hardware issue is detected.
    Generally will raise if the device's controller cannot read the physical sensor itself.
    """
    HARDWARE_FAULT_COLOR = 0x20
    """The hardware fault flag corresponding to the color sensor IC, which will raise if a hardware issue is detected.
    Generally will raise if the device's controller cannot read the physical sensor itself.
    """
    I2C_BUS_RECOVERY = 0x40
    """The I2C bus recovery flag, which will raise when the device needs to completely restart the I2C bus.
    This fault flag should not be active for very long; if it is stuck as an active fault, that may indicate a hardware issue.
    """

class DigoutCond(enum.Flag):
    SLOT0 = 0x1
    """Slot 0"""
    SLOT1 = 0x2
    """Slot 1"""
    SLOT2 = 0x4
    """Slot 2"""
    SLOT3 = 0x8
    """Slot 3"""
    SLOT4 = 0x10
    """Slot 4"""
    SLOT5 = 0x20
    """Slot 5"""
    SLOT6 = 0x40
    """Slot 6"""
    SLOT7 = 0x80
    """Slot 7"""
    SLOT8 = 0x100
    """Slot 8"""
    SLOT9 = 0x200
    """Slot 9"""
    SLOT10 = 0x400
    """Slot 10"""
    SLOT11 = 0x800
    """Slot 11"""
    SLOT12 = 0x1000
    """Slot 12"""
    SLOT13 = 0x2000
    """Slot 13"""
    SLOT14 = 0x4000
    """Slot 14"""
    SLOT15 = 0x8000
    """Slot 15"""


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
class DigoutControlConfig:
    output_config: Annotated[DigoutOutputConfig, Signal(0, Enum(width=8, dtype=DigoutOutputConfig, default_value=DigoutOutputConfig.DISABLED))]
    """Enable digout pad"""
    pwm_data_source: Annotated[DataSource, Signal(8, Enum(width=4, dtype=DataSource, default_value=DataSource.ZERO))]
    """The data source to use in PWM mode."""


@dataclasses.dataclass
class DigoutMessageTrigger:
    positive_edge: Annotated[bool, Signal(0, Boolean(False))]
    """Send digout message on positive edge (false->true)"""
    negative_edge: Annotated[bool, Signal(1, Boolean(False))]
    """Send digout message on negative edge (true->false)"""


@dataclasses.dataclass
class DigoutSlot:
    slot_enabled: Annotated[bool, Signal(0, Boolean(False))]
    """Enable the digout slot"""
    next_slot_action: Annotated[NextSlotAction, Signal(1, Enum(width=2, dtype=NextSlotAction, default_value=NextSlotAction.TERMINATE_CHAIN))]
    """How the digout slot interacts with the next slot"""
    invert_value: Annotated[bool, Signal(3, Boolean(False))]
    """Invert the digout slot's boolean value"""
    opcode: Annotated[SlotOpcode, Signal(4, Enum(width=7, dtype=SlotOpcode, default_value=SlotOpcode.EQUALS))]
    """Opcode"""
    immidiate_additive: Annotated[int, Signal(11, SInt(width=21, min=-1048576, max=1048575, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Additive immidiate"""
    immidiate_scaling: Annotated[int, Signal(32, UInt(width=8, min=0, max=255, default_value=255, factor_num=1, factor_den=256, offset=0))]
    """Scaling immidiate"""
    data_source_a: Annotated[DataSource, Signal(40, Enum(width=4, dtype=DataSource, default_value=DataSource.ZERO))]
    """First ``LHS`` data source"""
    data_source_b: Annotated[DataSource, Signal(44, Enum(width=4, dtype=DataSource, default_value=DataSource.ZERO))]
    """Second ``RHS`` data source"""

