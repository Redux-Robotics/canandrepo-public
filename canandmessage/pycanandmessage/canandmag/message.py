
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *


@dataclasses.dataclass
class CanIdArbitrate(BaseMessage):
    """select conflicting device to use"""
    __meta__ = MessageMeta(device_type=7, id=0, min_length=8, max_length=8)
    addr_value: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """Value corresponding to what was broadcasted in the CAN_ID_ERROR packet"""



@dataclasses.dataclass
class CanIdError(BaseMessage):
    """can id conflict tx packet"""
    __meta__ = MessageMeta(device_type=7, id=1, min_length=8, max_length=8)
    addr_value: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """Device-unique value that can be used during arbitration"""



@dataclasses.dataclass
class SettingCommand(BaseMessage):
    """setting control command"""
    __meta__ = MessageMeta(device_type=7, id=2, min_length=1, max_length=8)
    control_flag: Annotated[device_types.SettingCommand, Signal(0, Enum(width=8, dtype=device_types.SettingCommand, default_value=0))]
    """Setting command index"""
    setting_index: Annotated[Optional[device_types.Setting], Signal(8, Enum(width=8, dtype=device_types.Setting, default_value=0), optional=True)]
    """setting index to fetch"""



@dataclasses.dataclass
class SetSetting(BaseMessage):
    """update setting on device"""
    __meta__ = MessageMeta(device_type=7, id=3, min_length=8, max_length=8)
    address: Annotated[device_types.Setting, Signal(0, Enum(width=8, dtype=device_types.Setting, default_value=0))]
    """Setting index to write to"""
    value: Annotated[bytearray, Signal(8, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """6-byte setting value"""
    flags: Annotated[device_types.SettingFlags, Signal(56, Struct(device_types.SettingFlags))]
    """Setting flags"""



@dataclasses.dataclass
class ReportSetting(BaseMessage):
    """setting value report from device"""
    __meta__ = MessageMeta(device_type=7, id=4, min_length=8, max_length=8)
    address: Annotated[device_types.Setting, Signal(0, Enum(width=8, dtype=device_types.Setting, default_value=0))]
    """Setting index to write to"""
    value: Annotated[bytearray, Signal(8, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """6-byte setting value"""
    flags: Annotated[device_types.SettingReportFlags, Signal(56, Bitset(width=8, dtype=device_types.SettingReportFlags, default_value=0))]
    """Setting receive status"""



@dataclasses.dataclass
class ClearStickyFaults(BaseMessage):
    """Clear device sticky faults"""
    __meta__ = MessageMeta(device_type=7, id=5, min_length=0, max_length=8)




@dataclasses.dataclass
class Status(BaseMessage):
    """Status frame"""
    __meta__ = MessageMeta(device_type=7, id=6, min_length=8, max_length=8)
    faults: Annotated[device_types.Faults, Signal(0, Bitset(width=8, dtype=device_types.Faults, default_value=0))]
    """8-bit active faults bitfield"""
    sticky_faults: Annotated[device_types.Faults, Signal(8, Bitset(width=8, dtype=device_types.Faults, default_value=0))]
    """8-bit sticky faults bitfield"""
    temperature: Annotated[int, Signal(16, SInt(width=8, min=-128, max=127, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """8-bit signed temperature byte in Celsius"""



@dataclasses.dataclass
class PartyMode(BaseMessage):
    """Party mode"""
    __meta__ = MessageMeta(device_type=7, id=7, min_length=1, max_length=8)
    party_level: Annotated[int, Signal(0, UInt(width=8, min=0, max=255, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Party level. 0 disables the strobe, whereas 1 enables it."""



@dataclasses.dataclass
class OtaData(BaseMessage):
    """Firmware update payload"""
    __meta__ = MessageMeta(device_type=7, id=8, min_length=8, max_length=8)
    data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA data"""



@dataclasses.dataclass
class OtaToHost(BaseMessage):
    """Firmware update response."""
    __meta__ = MessageMeta(device_type=7, id=9, min_length=8, max_length=8)
    to_host_data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA to host data (dlc may vary)"""



@dataclasses.dataclass
class OtaToDevice(BaseMessage):
    """Firmware update command."""
    __meta__ = MessageMeta(device_type=7, id=10, min_length=8, max_length=8)
    to_device_data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA to device data (dlc may vary)"""



@dataclasses.dataclass
class Enumerate(BaseMessage):
    """Device enumerate response"""
    __meta__ = MessageMeta(device_type=7, id=11, min_length=8, max_length=8)
    serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Device-unique serial number"""
    is_bootloader: Annotated[bool, Signal(48, Boolean(False))]
    """Device is in bootloader."""



@dataclasses.dataclass
class AtomicBondAnnouncement(BaseMessage):
    """Atomic bond announcement. Sent by gateway to control bus state, and by devices during negotiation."""
    __meta__ = MessageMeta(device_type=7, id=12, min_length=8, max_length=8)
    gateway_serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Gateway's unique serial number"""
    flags: Annotated[device_types.AtomicAnnouncementFlags, Signal(48, Bitset(width=8, dtype=device_types.AtomicAnnouncementFlags, default_value=0))]
    """Announcement Flags"""
    rate: Annotated[device_types.AtomicBondBusRate, Signal(56, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """New bus rate for initialization"""



@dataclasses.dataclass
class AtomicBondSpecification(BaseMessage):
    """Atomic bond specification. Sent by devices to announce capabilities."""
    __meta__ = MessageMeta(device_type=7, id=13, min_length=8, max_length=8)
    device_serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Device's unique serial number"""
    max_supported_rate: Annotated[device_types.AtomicBondBusRate, Signal(48, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """Supported bus rates"""
    current_rate: Annotated[device_types.AtomicBondBusRate, Signal(56, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """Current bus rate, if confirming"""



@dataclasses.dataclass
class PositionOutput(BaseMessage):
    """Position frame"""
    __meta__ = MessageMeta(device_type=7, id=31, min_length=6, max_length=6)
    relative_position: Annotated[int, Signal(0, SInt(width=32, min=-2147483648, max=2147483647, default_value=0, factor_num=1, factor_den=16384, offset=0))]
    """32-bit signed relative position in 1/16384-ths of a rotation. This value does not persist on reboots."""
    magnet_status: Annotated[int, Signal(32, UInt(width=2, min=0, max=3, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """2-bit magnet status. If both bits are zero, the magnet is in range."""
    absolute_position: Annotated[int, Signal(34, UInt(width=14, min=0, max=16383, default_value=0, factor_num=1, factor_den=16384, offset=0))]
    """14-bit unsigned absolute position in 1/16384-ths of a rotation. The zero offset of the absolute encoder will preserve through reboots."""



@dataclasses.dataclass
class VelocityOutput(BaseMessage):
    """Velocity frame"""
    __meta__ = MessageMeta(device_type=7, id=30, min_length=3, max_length=3)
    velocity: Annotated[int, Signal(0, SInt(width=22, min=-2097152, max=2097151, default_value=0, factor_num=1, factor_den=1024, offset=0))]
    """Velocity as a 22-bit signed integer. One velocity tick corresponds to 1/1024th of a rotation per second."""
    magnet_status: Annotated[int, Signal(22, UInt(width=2, min=0, max=3, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """2-bit magnet status. If both bits are zero, the magnet is in range."""



@dataclasses.dataclass
class RawPositionOutput(BaseMessage):
    """Raw position frame"""
    __meta__ = MessageMeta(device_type=7, id=29, min_length=6, max_length=6)
    raw_position: Annotated[int, Signal(0, UInt(width=14, min=0, max=16383, default_value=0, factor_num=1, factor_den=16384, offset=0))]
    """14-bit raw absolute position in 1/16384-ths of a rotation."""
    magnet_status: Annotated[int, Signal(14, UInt(width=2, min=0, max=3, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """2-bit magnet status. If both bits are zero, the magnet is in range."""
    timestamp: Annotated[int, Signal(16, UInt(width=32, min=0, max=4294967295, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """32-bit sensor reading timestamp in microseconds since device boot."""


__all__ = ['MessageType', 'CanIdArbitrate', 'CanIdError', 'SettingCommand', 'SetSetting', 'ReportSetting', 'ClearStickyFaults', 'Status', 'PartyMode', 'OtaData', 'OtaToHost', 'OtaToDevice', 'Enumerate', 'AtomicBondAnnouncement', 'AtomicBondSpecification', 'PositionOutput', 'VelocityOutput', 'RawPositionOutput']

type MessageType = CanIdArbitrate | CanIdError | SettingCommand | SetSetting | ReportSetting | ClearStickyFaults | Status | PartyMode | OtaData | OtaToHost | OtaToDevice | Enumerate | AtomicBondAnnouncement | AtomicBondSpecification | PositionOutput | VelocityOutput | RawPositionOutput