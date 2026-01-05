
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *


@dataclasses.dataclass
class CanIdArbitrate(BaseMessage):
    """select conflicting device to use"""
    __meta__ = MessageMeta(device_type=4, id=0, min_length=8, max_length=8)
    addr_value: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """Value corresponding to what was broadcasted in the CAN_ID_ERROR packet"""



@dataclasses.dataclass
class CanIdError(BaseMessage):
    """can id conflict tx packet"""
    __meta__ = MessageMeta(device_type=4, id=1, min_length=8, max_length=8)
    addr_value: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """Device-unique value that can be used during arbitration"""



@dataclasses.dataclass
class SettingCommand(BaseMessage):
    """setting control command"""
    __meta__ = MessageMeta(device_type=4, id=2, min_length=1, max_length=8)
    control_flag: Annotated[device_types.SettingCommand, Signal(0, Enum(width=8, dtype=device_types.SettingCommand, default_value=0))]
    """Setting command index"""
    setting_index: Annotated[Optional[device_types.Setting], Signal(8, Enum(width=8, dtype=device_types.Setting, default_value=0), optional=True)]
    """setting index to fetch"""



@dataclasses.dataclass
class SetSetting(BaseMessage):
    """update setting on device"""
    __meta__ = MessageMeta(device_type=4, id=3, min_length=8, max_length=8)
    address: Annotated[device_types.Setting, Signal(0, Enum(width=8, dtype=device_types.Setting, default_value=0))]
    """Setting index to write to"""
    value: Annotated[bytearray, Signal(8, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """6-byte setting value"""
    flags: Annotated[device_types.SettingFlags, Signal(56, Struct(device_types.SettingFlags))]
    """Setting flags"""



@dataclasses.dataclass
class ReportSetting(BaseMessage):
    """setting value report from device"""
    __meta__ = MessageMeta(device_type=4, id=4, min_length=8, max_length=8)
    address: Annotated[device_types.Setting, Signal(0, Enum(width=8, dtype=device_types.Setting, default_value=0))]
    """Setting index to write to"""
    value: Annotated[bytearray, Signal(8, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """6-byte setting value"""
    flags: Annotated[device_types.SettingReportFlags, Signal(56, Bitset(width=8, dtype=device_types.SettingReportFlags, default_value=0))]
    """Setting receive status"""



@dataclasses.dataclass
class ClearStickyFaults(BaseMessage):
    """Clear device sticky faults"""
    __meta__ = MessageMeta(device_type=4, id=5, min_length=0, max_length=8)




@dataclasses.dataclass
class Status(BaseMessage):
    """Status frame"""
    __meta__ = MessageMeta(device_type=4, id=6, min_length=8, max_length=8)
    faults: Annotated[device_types.Faults, Signal(0, Bitset(width=8, dtype=device_types.Faults, default_value=0))]
    """8-bit active faults bitfield"""
    sticky_faults: Annotated[device_types.Faults, Signal(8, Bitset(width=8, dtype=device_types.Faults, default_value=0))]
    """8-bit sticky faults bitfield"""
    temperature: Annotated[int, Signal(16, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=1, factor_den=256, offset=0))]
    """16-bit signed temperature byte in 1/256ths of a Celsius"""



@dataclasses.dataclass
class PartyMode(BaseMessage):
    """Party mode"""
    __meta__ = MessageMeta(device_type=4, id=7, min_length=1, max_length=8)
    party_level: Annotated[int, Signal(0, UInt(width=8, min=0, max=255, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Party level. 0 disables the strobe, whereas 1 enables it."""



@dataclasses.dataclass
class OtaData(BaseMessage):
    """Firmware update payload"""
    __meta__ = MessageMeta(device_type=4, id=8, min_length=8, max_length=8)
    data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA data"""



@dataclasses.dataclass
class OtaToHost(BaseMessage):
    """Firmware update response."""
    __meta__ = MessageMeta(device_type=4, id=9, min_length=8, max_length=8)
    to_host_data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA to host data (dlc may vary)"""



@dataclasses.dataclass
class OtaToDevice(BaseMessage):
    """Firmware update command."""
    __meta__ = MessageMeta(device_type=4, id=10, min_length=8, max_length=8)
    to_device_data: Annotated[bytearray, Signal(0, Buffer(width=64, default_value=b'\x00\x00\x00\x00\x00\x00\x00\x00'))]
    """OTA to device data (dlc may vary)"""



@dataclasses.dataclass
class Enumerate(BaseMessage):
    """Device enumerate response"""
    __meta__ = MessageMeta(device_type=4, id=11, min_length=8, max_length=8)
    serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Device-unique serial number"""
    is_bootloader: Annotated[bool, Signal(48, Boolean(False))]
    """Device is in bootloader."""



@dataclasses.dataclass
class AtomicBondAnnouncement(BaseMessage):
    """Atomic bond announcement. Sent by gateway to control bus state, and by devices during negotiation."""
    __meta__ = MessageMeta(device_type=4, id=12, min_length=8, max_length=8)
    gateway_serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Gateway's unique serial number"""
    flags: Annotated[device_types.AtomicAnnouncementFlags, Signal(48, Bitset(width=8, dtype=device_types.AtomicAnnouncementFlags, default_value=0))]
    """Announcement Flags"""
    rate: Annotated[device_types.AtomicBondBusRate, Signal(56, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """New bus rate for initialization"""



@dataclasses.dataclass
class AtomicBondSpecification(BaseMessage):
    """Atomic bond specification. Sent by devices to announce capabilities."""
    __meta__ = MessageMeta(device_type=4, id=13, min_length=8, max_length=8)
    device_serial: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Device's unique serial number"""
    max_supported_rate: Annotated[device_types.AtomicBondBusRate, Signal(48, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """Supported bus rates"""
    current_rate: Annotated[device_types.AtomicBondBusRate, Signal(56, Enum(width=8, dtype=device_types.AtomicBondBusRate, default_value=device_types.AtomicBondBusRate.RATE_1M_2B))]
    """Current bus rate, if confirming"""



@dataclasses.dataclass
class YawOutput(BaseMessage):
    """Yaw angle frame"""
    __meta__ = MessageMeta(device_type=4, id=31, min_length=6, max_length=6)
    yaw: Annotated[device_types.Yaw, Signal(0, Struct(device_types.Yaw))]
    """Yaw value"""



@dataclasses.dataclass
class AngularPositionOutput(BaseMessage):
    """Angular position quaternion frame"""
    __meta__ = MessageMeta(device_type=4, id=30, min_length=8, max_length=8)
    w: Annotated[int, Signal(0, SInt(width=16, min=-32767, max=32767, default_value=0, factor_num=1, factor_den=32767, offset=0))]
    """Quaternion w term"""
    x: Annotated[int, Signal(16, SInt(width=16, min=-32767, max=32767, default_value=0, factor_num=1, factor_den=32767, offset=0))]
    """Quaternion x term"""
    y: Annotated[int, Signal(32, SInt(width=16, min=-32767, max=32767, default_value=0, factor_num=1, factor_den=32767, offset=0))]
    """Quaternion y term"""
    z: Annotated[int, Signal(48, SInt(width=16, min=-32767, max=32767, default_value=0, factor_num=1, factor_den=32767, offset=0))]
    """Quaternion z term"""



@dataclasses.dataclass
class AngularVelocityOutput(BaseMessage):
    """Angular velocity frame"""
    __meta__ = MessageMeta(device_type=4, id=29, min_length=6, max_length=6)
    yaw: Annotated[int, Signal(0, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=2000, factor_den=32767, offset=0))]
    """Yaw velocity"""
    pitch: Annotated[int, Signal(16, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=2000, factor_den=32767, offset=0))]
    """Pitch velocity"""
    roll: Annotated[int, Signal(32, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=2000, factor_den=32767, offset=0))]
    """Roll velocity"""



@dataclasses.dataclass
class AccelerationOutput(BaseMessage):
    """Acceleration frame"""
    __meta__ = MessageMeta(device_type=4, id=28, min_length=6, max_length=6)
    z: Annotated[int, Signal(0, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=1, factor_den=2048, offset=0))]
    """Z-axis acceleration"""
    y: Annotated[int, Signal(16, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=1, factor_den=2048, offset=0))]
    """Y-axis acceleration"""
    x: Annotated[int, Signal(32, SInt(width=16, min=-32768, max=32767, default_value=0, factor_num=1, factor_den=2048, offset=0))]
    """X-axis acceleration"""



@dataclasses.dataclass
class Calibrate(BaseMessage):
    """Trigger Calibration"""
    __meta__ = MessageMeta(device_type=4, id=27, min_length=8, max_length=8)
    calibration_type: Annotated[device_types.CalibrationType, Signal(0, Enum(width=8, dtype=device_types.CalibrationType, default_value=device_types.CalibrationType.NORMAL))]
    """Calibration type"""



@dataclasses.dataclass
class CalibrationStatus(BaseMessage):
    """Calibration Status"""
    __meta__ = MessageMeta(device_type=4, id=26, min_length=8, max_length=8)



__all__ = ['MessageType', 'CanIdArbitrate', 'CanIdError', 'SettingCommand', 'SetSetting', 'ReportSetting', 'ClearStickyFaults', 'Status', 'PartyMode', 'OtaData', 'OtaToHost', 'OtaToDevice', 'Enumerate', 'AtomicBondAnnouncement', 'AtomicBondSpecification', 'YawOutput', 'AngularPositionOutput', 'AngularVelocityOutput', 'AccelerationOutput', 'Calibrate', 'CalibrationStatus']

type MessageType = CanIdArbitrate | CanIdError | SettingCommand | SetSetting | ReportSetting | ClearStickyFaults | Status | PartyMode | OtaData | OtaToHost | OtaToDevice | Enumerate | AtomicBondAnnouncement | AtomicBondSpecification | YawOutput | AngularPositionOutput | AngularVelocityOutput | AccelerationOutput | Calibrate | CalibrationStatus