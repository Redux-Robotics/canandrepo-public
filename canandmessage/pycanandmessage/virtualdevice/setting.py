
import dataclasses
import math
from typing import Optional, Annotated
from . import types as device_types
from . import message
from pycanandmessage.model import *

__all__ = ['SettingType', 'CanId', 'Name0', 'Name1', 'Name2', 'StatusFramePeriod', 'SerialNumber', 'FirmwareVersion', 'ChickenBits', 'DeviceType', 'Scratch0', 'Scratch1']


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
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'Device'))]
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
    value: Annotated[int, Signal(0, UInt(width=16, min=1, max=16383, default_value=100, factor_num=1, factor_den=1000, offset=0))]
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


type SettingType = CanId | Name0 | Name1 | Name2 | StatusFramePeriod | SerialNumber | FirmwareVersion | ChickenBits | DeviceType | Scratch0 | Scratch1
