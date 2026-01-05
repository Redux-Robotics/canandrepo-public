
import dataclasses
import math
from typing import Optional, Annotated
from . import types as device_types
from . import message
from pycanandmessage.model import *

__all__ = ['SettingType', 'CanId', 'Name0', 'Name1', 'Name2', 'StatusFramePeriod', 'SerialNumber', 'FirmwareVersion', 'ChickenBits', 'DeviceType', 'Scratch0', 'Scratch1', 'DistanceFramePeriod', 'ColorFramePeriod', 'DigoutFramePeriod', 'DistanceExtraFrameMode', 'ColorExtraFrameMode', 'LampBrightness', 'ColorIntegrationPeriod', 'DistanceIntegrationPeriod', 'Digout1OutputConfig', 'Digout2OutputConfig', 'Digout1MessageOnChange', 'Digout2MessageOnChange', 'Digout1Config0', 'Digout1Config1', 'Digout1Config2', 'Digout1Config3', 'Digout1Config4', 'Digout1Config5', 'Digout1Config6', 'Digout1Config7', 'Digout1Config8', 'Digout1Config9', 'Digout1Config10', 'Digout1Config11', 'Digout1Config12', 'Digout1Config13', 'Digout1Config14', 'Digout1Config15', 'Digout2Config0', 'Digout2Config1', 'Digout2Config2', 'Digout2Config3', 'Digout2Config4', 'Digout2Config5', 'Digout2Config6', 'Digout2Config7', 'Digout2Config8', 'Digout2Config9', 'Digout2Config10', 'Digout2Config11', 'Digout2Config12', 'Digout2Config13', 'Digout2Config14', 'Digout2Config15']


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
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'color\x00'))]
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


@dataclasses.dataclass
class DistanceFramePeriod(BaseSetting):
    """Distance frame period (ms)"""
    __meta__ = SettingMeta(idx=0xff, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=20, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class ColorFramePeriod(BaseSetting):
    """Color frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfe, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=25, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class DigoutFramePeriod(BaseSetting):
    """Digout frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfd, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=100, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class DistanceExtraFrameMode(BaseSetting):
    """Distance extra frame mode"""
    __meta__ = SettingMeta(idx=0xf7, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.ExtraFrameMode, Signal(0, Enum(width=8, dtype=device_types.ExtraFrameMode, default_value=device_types.ExtraFrameMode.EARLY_TRANSMIT_ON_CHANGE))]
    """Setting value"""


@dataclasses.dataclass
class ColorExtraFrameMode(BaseSetting):
    """Color extra frame frame mode"""
    __meta__ = SettingMeta(idx=0xf6, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.ExtraFrameMode, Signal(0, Enum(width=8, dtype=device_types.ExtraFrameMode, default_value=device_types.ExtraFrameMode.EARLY_TRANSMIT_ON_CHANGE))]
    """Setting value"""


@dataclasses.dataclass
class LampBrightness(BaseSetting):
    """Lamp LED brightness"""
    __meta__ = SettingMeta(idx=0xef, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=36000, default_value=36000, factor_num=1, factor_den=36000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class ColorIntegrationPeriod(BaseSetting):
    """Color integration period"""
    __meta__ = SettingMeta(idx=0xee, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.ColorIntegrationPeriod, Signal(0, Enum(width=4, dtype=device_types.ColorIntegrationPeriod, default_value=device_types.ColorIntegrationPeriod.PERIOD_25_ms_RESOLUTION_16_bit))]
    """Setting value"""


@dataclasses.dataclass
class DistanceIntegrationPeriod(BaseSetting):
    """Distance integration period"""
    __meta__ = SettingMeta(idx=0xed, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DistanceIntegrationPeriod, Signal(0, Enum(width=4, dtype=device_types.DistanceIntegrationPeriod, default_value=device_types.DistanceIntegrationPeriod.PERIOD_20_ms))]
    """Setting value"""


@dataclasses.dataclass
class Digout1OutputConfig(BaseSetting):
    """Digital output 1 control config"""
    __meta__ = SettingMeta(idx=0xeb, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutControlConfig, Signal(0, Struct(device_types.DigoutControlConfig))]
    """Setting value"""


@dataclasses.dataclass
class Digout2OutputConfig(BaseSetting):
    """Digital output 2 control config"""
    __meta__ = SettingMeta(idx=0xea, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutControlConfig, Signal(0, Struct(device_types.DigoutControlConfig))]
    """Setting value"""


@dataclasses.dataclass
class Digout1MessageOnChange(BaseSetting):
    """Digital output 1 send message on change"""
    __meta__ = SettingMeta(idx=0xe9, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutMessageTrigger, Signal(0, Struct(device_types.DigoutMessageTrigger))]
    """Setting value"""


@dataclasses.dataclass
class Digout2MessageOnChange(BaseSetting):
    """Digital output 2 send message on change"""
    __meta__ = SettingMeta(idx=0xe8, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutMessageTrigger, Signal(0, Struct(device_types.DigoutMessageTrigger))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config0(BaseSetting):
    """Digout1 config slot 0"""
    __meta__ = SettingMeta(idx=0xd0, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config1(BaseSetting):
    """Digout1 config slot 1"""
    __meta__ = SettingMeta(idx=0xcf, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config2(BaseSetting):
    """Digout1 config slot 2"""
    __meta__ = SettingMeta(idx=0xce, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config3(BaseSetting):
    """Digout1 config slot 3"""
    __meta__ = SettingMeta(idx=0xcd, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config4(BaseSetting):
    """Digout1 config slot 4"""
    __meta__ = SettingMeta(idx=0xcc, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config5(BaseSetting):
    """Digout1 config slot 5"""
    __meta__ = SettingMeta(idx=0xcb, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config6(BaseSetting):
    """Digout1 config slot 6"""
    __meta__ = SettingMeta(idx=0xca, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config7(BaseSetting):
    """Digout1 config slot 7"""
    __meta__ = SettingMeta(idx=0xc9, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config8(BaseSetting):
    """Digout1 config slot 8"""
    __meta__ = SettingMeta(idx=0xc8, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config9(BaseSetting):
    """Digout1 config slot 9"""
    __meta__ = SettingMeta(idx=0xc7, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config10(BaseSetting):
    """Digout1 config slot 10"""
    __meta__ = SettingMeta(idx=0xc6, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config11(BaseSetting):
    """Digout1 config slot 11"""
    __meta__ = SettingMeta(idx=0xc5, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config12(BaseSetting):
    """Digout1 config slot 12"""
    __meta__ = SettingMeta(idx=0xc4, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config13(BaseSetting):
    """Digout1 config slot 13"""
    __meta__ = SettingMeta(idx=0xc3, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config14(BaseSetting):
    """Digout1 config slot 14"""
    __meta__ = SettingMeta(idx=0xc2, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout1Config15(BaseSetting):
    """Digout1 config slot 15"""
    __meta__ = SettingMeta(idx=0xc1, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config0(BaseSetting):
    """Digout2 config slot 0"""
    __meta__ = SettingMeta(idx=0xc0, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config1(BaseSetting):
    """Digout2 config slot 1"""
    __meta__ = SettingMeta(idx=0xbf, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config2(BaseSetting):
    """Digout2 config slot 2"""
    __meta__ = SettingMeta(idx=0xbe, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config3(BaseSetting):
    """Digout2 config slot 3"""
    __meta__ = SettingMeta(idx=0xbd, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config4(BaseSetting):
    """Digout2 config slot 4"""
    __meta__ = SettingMeta(idx=0xbc, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config5(BaseSetting):
    """Digout2 config slot 5"""
    __meta__ = SettingMeta(idx=0xbb, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config6(BaseSetting):
    """Digout2 config slot 6"""
    __meta__ = SettingMeta(idx=0xba, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config7(BaseSetting):
    """Digout2 config slot 7"""
    __meta__ = SettingMeta(idx=0xb9, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config8(BaseSetting):
    """Digout2 config slot 8"""
    __meta__ = SettingMeta(idx=0xb8, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config9(BaseSetting):
    """Digout2 config slot 9"""
    __meta__ = SettingMeta(idx=0xb7, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config10(BaseSetting):
    """Digout2 config slot 10"""
    __meta__ = SettingMeta(idx=0xb6, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config11(BaseSetting):
    """Digout2 config slot 11"""
    __meta__ = SettingMeta(idx=0xb5, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config12(BaseSetting):
    """Digout2 config slot 12"""
    __meta__ = SettingMeta(idx=0xb4, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config13(BaseSetting):
    """Digout2 config slot 13"""
    __meta__ = SettingMeta(idx=0xb3, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config14(BaseSetting):
    """Digout2 config slot 14"""
    __meta__ = SettingMeta(idx=0xb2, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


@dataclasses.dataclass
class Digout2Config15(BaseSetting):
    """Digout2 config slot 15"""
    __meta__ = SettingMeta(idx=0xb1, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.DigoutSlot, Signal(0, Struct(device_types.DigoutSlot))]
    """Setting value"""


type SettingType = CanId | Name0 | Name1 | Name2 | StatusFramePeriod | SerialNumber | FirmwareVersion | ChickenBits | DeviceType | Scratch0 | Scratch1 | DistanceFramePeriod | ColorFramePeriod | DigoutFramePeriod | DistanceExtraFrameMode | ColorExtraFrameMode | LampBrightness | ColorIntegrationPeriod | DistanceIntegrationPeriod | Digout1OutputConfig | Digout2OutputConfig | Digout1MessageOnChange | Digout2MessageOnChange | Digout1Config0 | Digout1Config1 | Digout1Config2 | Digout1Config3 | Digout1Config4 | Digout1Config5 | Digout1Config6 | Digout1Config7 | Digout1Config8 | Digout1Config9 | Digout1Config10 | Digout1Config11 | Digout1Config12 | Digout1Config13 | Digout1Config14 | Digout1Config15 | Digout2Config0 | Digout2Config1 | Digout2Config2 | Digout2Config3 | Digout2Config4 | Digout2Config5 | Digout2Config6 | Digout2Config7 | Digout2Config8 | Digout2Config9 | Digout2Config10 | Digout2Config11 | Digout2Config12 | Digout2Config13 | Digout2Config14 | Digout2Config15
