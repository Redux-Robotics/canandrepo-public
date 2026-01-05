
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *


@dataclasses.dataclass
class CanIdArbitrate(BaseMessage):
    """select conflicting device to use"""
    __meta__ = MessageMeta(device_type=1, id=0, min_length=8, max_length=8)
    addr_value: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """Value corresponding to what was broadcasted in the CAN_ID_ERROR packet"""



@dataclasses.dataclass
class CanIdError(BaseMessage):
    """can id conflict tx packet"""
    __meta__ = MessageMeta(device_type=1, id=1, min_length=8, max_length=8)
    addr_value: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """Device-unique value that can be used during arbitration"""



@dataclasses.dataclass
class SettingCommand(BaseMessage):
    """setting control command"""
    __meta__ = MessageMeta(device_type=1, id=2, min_length=1, max_length=8)
    control_flag: Annotated[device_types.SettingCommand, Signal(0, Enum(width=8, dtype=device_types.SettingCommand, default_value=0))]
    """Setting command index"""
    setting_index: Annotated[Optional[device_types.Setting], Signal(8, Enum(width=8, dtype=device_types.Setting, default_value=0), optional=True)]
    """setting index to fetch"""



@dataclasses.dataclass
class SetSetting(BaseMessage):
    """update setting on device"""
    __meta__ = MessageMeta(device_type=1, id=3, min_length=8, max_length=8)
    address: Annotated[device_types.Setting, Signal(0, Enum(width=8, dtype=device_types.Setting, default_value=0))]
    """Setting index to write to"""
    value: Annotated[bytearray, Signal(8, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """6-byte setting value"""
    flags: Annotated[device_types.SettingFlags, Signal(56, Struct(device_types.SettingFlags))]
    """Setting flags"""



@dataclasses.dataclass
class ReportSetting(BaseMessage):
    """setting value report from device"""
    __meta__ = MessageMeta(device_type=1, id=4, min_length=8, max_length=8)
    address: Annotated[device_types.Setting, Signal(0, Enum(width=8, dtype=device_types.Setting, default_value=0))]
    """Setting index to write to"""
    value: Annotated[bytearray, Signal(8, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """6-byte setting value"""
    flags: Annotated[device_types.SettingReportFlags, Signal(56, Bitset(width=8, dtype=device_types.SettingReportFlags, default_value=0))]
    """Setting receive status"""



@dataclasses.dataclass
class ClearStickyFaults(BaseMessage):
    """Clear device sticky faults"""
    __meta__ = MessageMeta(device_type=1, id=5, min_length=0, max_length=8)




@dataclasses.dataclass
class Status(BaseMessage):
    """Status frame"""
    __meta__ = MessageMeta(device_type=1, id=6, min_length=8, max_length=8)
    dev_specific: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """Device-specific status data. See device pages for more information."""



@dataclasses.dataclass
class PartyMode(BaseMessage):
    """Party mode"""
    __meta__ = MessageMeta(device_type=1, id=7, min_length=1, max_length=8)
    party_level: Annotated[int, Signal(0, UInt(width=8, min=0, max=255, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Party level. 0 disables the strobe, whereas 1 enables it."""



@dataclasses.dataclass
class OtaData(BaseMessage):
    """Firmware update payload"""
    __meta__ = MessageMeta(device_type=1, id=8, min_length=8, max_length=8)
    data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA data"""



@dataclasses.dataclass
class OtaToHost(BaseMessage):
    """Firmware update response."""
    __meta__ = MessageMeta(device_type=1, id=9, min_length=8, max_length=8)
    to_host_data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA to host data (dlc may vary)"""



@dataclasses.dataclass
class OtaToDevice(BaseMessage):
    """Firmware update command."""
    __meta__ = MessageMeta(device_type=1, id=10, min_length=8, max_length=8)
    to_device_data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA to device data (dlc may vary)"""



@dataclasses.dataclass
class Enumerate(BaseMessage):
    """Device enumerate response"""
    __meta__ = MessageMeta(device_type=1, id=11, min_length=8, max_length=8)
    serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Device-unique serial number"""
    is_bootloader: Annotated[bool, Signal(48, Boolean(False))]
    """Device is in bootloader."""



@dataclasses.dataclass
class AtomicBondAnnouncement(BaseMessage):
    """Atomic bond announcement. Sent by gateway to control bus state, and by devices during negotiation."""
    __meta__ = MessageMeta(device_type=1, id=12, min_length=8, max_length=8)
    gateway_serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Gateway's unique serial number"""
    flags: Annotated[device_types.AtomicAnnouncementFlags, Signal(48, Bitset(width=8, dtype=device_types.AtomicAnnouncementFlags, default_value=0))]
    """Announcement Flags"""
    rate: Annotated[device_types.AtomicBondBusRate, Signal(56, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """New bus rate for initialization"""



@dataclasses.dataclass
class AtomicBondSpecification(BaseMessage):
    """Atomic bond specification. Sent by devices to announce capabilities."""
    __meta__ = MessageMeta(device_type=1, id=13, min_length=8, max_length=8)
    device_serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Device's unique serial number"""
    max_supported_rate: Annotated[device_types.AtomicBondBusRate, Signal(48, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """Supported bus rates"""
    current_rate: Annotated[device_types.AtomicBondBusRate, Signal(56, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """Current bus rate, if confirming"""



@dataclasses.dataclass
class DigitalValue(BaseMessage):
    """Digital value"""
    __meta__ = MessageMeta(device_type=1, id=31, min_length=2, max_length=2)
    dig0: Annotated[bool, Signal(0, Boolean(False))]
    """Digital value 0"""
    dig1: Annotated[bool, Signal(1, Boolean(False))]
    """Digital value 1"""
    dig2: Annotated[bool, Signal(2, Boolean(False))]
    """Digital value 2"""
    dig3: Annotated[bool, Signal(3, Boolean(False))]
    """Digital value 3"""
    dig4: Annotated[bool, Signal(4, Boolean(False))]
    """Digital value 4"""
    dig5: Annotated[bool, Signal(5, Boolean(False))]
    """Digital value 5"""
    dig6: Annotated[bool, Signal(6, Boolean(False))]
    """Digital value 6"""
    dig7: Annotated[bool, Signal(7, Boolean(False))]
    """Digital value 7"""
    dig8: Annotated[bool, Signal(8, Boolean(False))]
    """Digital value 0"""
    dig9: Annotated[bool, Signal(9, Boolean(False))]
    """Digital value 10"""
    dig10: Annotated[bool, Signal(10, Boolean(False))]
    """Digital value 11"""
    dig11: Annotated[bool, Signal(11, Boolean(False))]
    """Digital value 12"""
    dig12: Annotated[bool, Signal(12, Boolean(False))]
    """Digital value 13"""
    dig13: Annotated[bool, Signal(13, Boolean(False))]
    """Digital value 14"""
    dig14: Annotated[bool, Signal(14, Boolean(False))]
    """Digital value 15"""
    dig15: Annotated[bool, Signal(15, Boolean(False))]
    """Digital value 16"""



@dataclasses.dataclass
class GyroValue(BaseMessage):
    """Gyroscope rotational data"""
    __meta__ = MessageMeta(device_type=1, id=30, min_length=8, max_length=8)
    position: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Position (rotations)"""
    velocity: Annotated[float, Signal(32, Float(width=32, min=None, max=None, default_value=0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Velocity (rotations per second)"""


__all__ = ['MessageType', 'CanIdArbitrate', 'CanIdError', 'SettingCommand', 'SetSetting', 'ReportSetting', 'ClearStickyFaults', 'Status', 'PartyMode', 'OtaData', 'OtaToHost', 'OtaToDevice', 'Enumerate', 'AtomicBondAnnouncement', 'AtomicBondSpecification', 'DigitalValue', 'GyroValue']

type MessageType = CanIdArbitrate | CanIdError | SettingCommand | SetSetting | ReportSetting | ClearStickyFaults | Status | PartyMode | OtaData | OtaToHost | OtaToDevice | Enumerate | AtomicBondAnnouncement | AtomicBondSpecification | DigitalValue | GyroValue