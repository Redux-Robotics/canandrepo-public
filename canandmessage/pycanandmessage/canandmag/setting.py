
import dataclasses
import math
from typing import Optional, Annotated
from . import types as device_types
from . import message
from pycanandmessage.model import *

__all__ = ['SettingType', 'CanId', 'Name0', 'Name1', 'Name2', 'StatusFramePeriod', 'SerialNumber', 'FirmwareVersion', 'ChickenBits', 'DeviceType', 'Scratch0', 'Scratch1', 'ZeroOffset', 'VelocityWindow', 'PositionFramePeriod', 'VelocityFramePeriod', 'RawPositionFramePeriod', 'InvertDirection', 'RelativePosition', 'DisableZeroButton']


@dataclasses.dataclass
class CanId(BaseSetting):
    """CAN Device ID"""
    __meta__ = SettingMeta(idx=0x0, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=8, min=0, max=63, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class Name0(BaseSetting):
    """device_name[0:5]"""
    __meta__ = SettingMeta(idx=0x1, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'Canand'))]
    """Setting value"""


@dataclasses.dataclass
class Name1(BaseSetting):
    """device_name[6:11]"""
    __meta__ = SettingMeta(idx=0x2, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'mag\x00\x00\x00'))]
    """Setting value"""


@dataclasses.dataclass
class Name2(BaseSetting):
    """device_name[12:17]"""
    __meta__ = SettingMeta(idx=0x3, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Setting value"""


@dataclasses.dataclass
class StatusFramePeriod(BaseSetting):
    """Status frame period (ms)"""
    __meta__ = SettingMeta(idx=0x4, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=100, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class SerialNumber(BaseSetting):
    """Serial number"""
    __meta__ = SettingMeta(idx=0x5, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Setting value"""


@dataclasses.dataclass
class FirmwareVersion(BaseSetting):
    """Firmware version"""
    __meta__ = SettingMeta(idx=0x6, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.FirmwareVersion, Signal(0, Struct(device_types.FirmwareVersion))]
    """Setting value"""


@dataclasses.dataclass
class ChickenBits(BaseSetting):
    """Device-specific chicken bits"""
    __meta__ = SettingMeta(idx=0x7, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Setting value"""


@dataclasses.dataclass
class DeviceType(BaseSetting):
    """Device-specific type identifier"""
    __meta__ = SettingMeta(idx=0x8, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=0, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class Scratch0(BaseSetting):
    """User-writable scratch bytes 1"""
    __meta__ = SettingMeta(idx=0x9, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Setting value"""


@dataclasses.dataclass
class Scratch1(BaseSetting):
    """User-writable scratch bytes 2"""
    __meta__ = SettingMeta(idx=0xa, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'\x00\x00\x00\x00\x00\x00'))]
    """Setting value"""


@dataclasses.dataclass
class ZeroOffset(BaseSetting):
    """Encoder zero offset"""
    __meta__ = SettingMeta(idx=0xff, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.ZeroOffset, Signal(0, Struct(device_types.ZeroOffset))]
    """Setting value"""


@dataclasses.dataclass
class VelocityWindow(BaseSetting):
    """Velocity window width (value*250us)"""
    __meta__ = SettingMeta(idx=0xfe, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=8, min=1, max=255, default_value=100, factor_num=1, factor_den=4, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class PositionFramePeriod(BaseSetting):
    """Position frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfd, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=20, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class VelocityFramePeriod(BaseSetting):
    """Velocity frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfc, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=20, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class RawPositionFramePeriod(BaseSetting):
    """Raw position frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfb, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=0, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class InvertDirection(BaseSetting):
    """Invert direction (0=ccw, 1=cw)"""
    __meta__ = SettingMeta(idx=0xfa, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bool, Signal(0, Boolean(False))]
    """Setting value"""


@dataclasses.dataclass
class RelativePosition(BaseSetting):
    """Set relative position value"""
    __meta__ = SettingMeta(idx=0xf9, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, SInt(width=32, min=-2147483648, max=2147483647, default_value=0, factor_num=1, factor_den=16384, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class DisableZeroButton(BaseSetting):
    """Disable the zero button"""
    __meta__ = SettingMeta(idx=0xf8, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[bool, Signal(0, Boolean(False))]
    """Setting value"""


type SettingType = CanId | Name0 | Name1 | Name2 | StatusFramePeriod | SerialNumber | FirmwareVersion | ChickenBits | DeviceType | Scratch0 | Scratch1 | ZeroOffset | VelocityWindow | PositionFramePeriod | VelocityFramePeriod | RawPositionFramePeriod | InvertDirection | RelativePosition | DisableZeroButton
