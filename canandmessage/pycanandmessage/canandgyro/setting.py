
import dataclasses
import math
from typing import Optional, Annotated
from . import types as device_types
from . import message
from pycanandmessage.model import *

__all__ = ['SettingType', 'CanId', 'Name0', 'Name1', 'Name2', 'StatusFramePeriod', 'SerialNumber', 'FirmwareVersion', 'ChickenBits', 'DeviceType', 'Scratch0', 'Scratch1', 'YawFramePeriod', 'AngularPositionFramePeriod', 'AngularVelocityFramePeriod', 'AccelerationFramePeriod', 'SetYaw', 'SetPosePositiveW', 'SetPoseNegativeW', 'GyroXSensitivity', 'GyroYSensitivity', 'GyroZSensitivity', 'GyroXZroOffset', 'GyroYZroOffset', 'GyroZZroOffset', 'GyroZroOffsetTemperature', 'TemperatureCalibrationX0', 'TemperatureCalibrationY0', 'TemperatureCalibrationZ0', 'TemperatureCalibrationT0', 'TemperatureCalibrationX1', 'TemperatureCalibrationY1', 'TemperatureCalibrationZ1', 'TemperatureCalibrationT1']


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
    value: Annotated[bytearray, Signal(0, Buffer(width=48, default_value=b'gyro\x00\x00'))]
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
class YawFramePeriod(BaseSetting):
    """Yaw angle frame period (ms)"""
    __meta__ = SettingMeta(idx=0xff, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=10, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class AngularPositionFramePeriod(BaseSetting):
    """Angular position frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfe, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=20, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class AngularVelocityFramePeriod(BaseSetting):
    """Angular velocity frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfd, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=100, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class AccelerationFramePeriod(BaseSetting):
    """Acceleration frame period (ms)"""
    __meta__ = SettingMeta(idx=0xfc, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[int, Signal(0, UInt(width=16, min=0, max=65535, default_value=100, factor_num=1, factor_den=1000, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class SetYaw(BaseSetting):
    """Set yaw"""
    __meta__ = SettingMeta(idx=0xfb, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.Yaw, Signal(0, Struct(device_types.Yaw))]
    """Setting value"""


@dataclasses.dataclass
class SetPosePositiveW(BaseSetting):
    """Set (normed) quaternion assuming positive W"""
    __meta__ = SettingMeta(idx=0xfa, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.QuatXyz, Signal(0, Struct(device_types.QuatXyz))]
    """Setting value"""


@dataclasses.dataclass
class SetPoseNegativeW(BaseSetting):
    """Set (normed) quaternion assuming negative W"""
    __meta__ = SettingMeta(idx=0xf9, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[device_types.QuatXyz, Signal(0, Struct(device_types.QuatXyz))]
    """Setting value"""


@dataclasses.dataclass
class GyroXSensitivity(BaseSetting):
    """Gyro X axis sensitivity"""
    __meta__ = SettingMeta(idx=0xf8, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=0.0, max=None, default_value=1.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class GyroYSensitivity(BaseSetting):
    """Gyro Y axis sensitivity"""
    __meta__ = SettingMeta(idx=0xf7, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=0.0, max=None, default_value=1.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class GyroZSensitivity(BaseSetting):
    """Gyro Z axis sensitivity"""
    __meta__ = SettingMeta(idx=0xf6, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=0.0, max=None, default_value=1.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class GyroXZroOffset(BaseSetting):
    """Gyro X-axis calibrated ZRO offset"""
    __meta__ = SettingMeta(idx=0xf5, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class GyroYZroOffset(BaseSetting):
    """Gyro Y-axis calibrated ZRO offset"""
    __meta__ = SettingMeta(idx=0xf4, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class GyroZZroOffset(BaseSetting):
    """Gyro Z-axis calibrated ZRO offset"""
    __meta__ = SettingMeta(idx=0xf3, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class GyroZroOffsetTemperature(BaseSetting):
    """Temperature at ZRO offset (celsius)"""
    __meta__ = SettingMeta(idx=0xf2, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=25.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationX0(BaseSetting):
    """Temp cal X-axis point 0"""
    __meta__ = SettingMeta(idx=0xe7, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationY0(BaseSetting):
    """Temp cal Y-axis point 0"""
    __meta__ = SettingMeta(idx=0xe6, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationZ0(BaseSetting):
    """Temp cal Z-axis point 0"""
    __meta__ = SettingMeta(idx=0xe5, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationT0(BaseSetting):
    """Temp cal temperature point 0 (celsius)"""
    __meta__ = SettingMeta(idx=0xe4, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationX1(BaseSetting):
    """Temp cal X-axis point 1"""
    __meta__ = SettingMeta(idx=0xe3, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationY1(BaseSetting):
    """Temp cal Y-axis point 1"""
    __meta__ = SettingMeta(idx=0xe2, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationZ1(BaseSetting):
    """Temp cal Z-axis point 1"""
    __meta__ = SettingMeta(idx=0xe1, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


@dataclasses.dataclass
class TemperatureCalibrationT1(BaseSetting):
    """Temp cal temperature point 1 (celsius)"""
    __meta__ = SettingMeta(idx=0xe0, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
    value: Annotated[float, Signal(0, Float(width=32, min=None, max=None, default_value=0.0, allow_nan_inf=False, factor_num=1, factor_den=1, offset=0))]
    """Setting value"""


type SettingType = CanId | Name0 | Name1 | Name2 | StatusFramePeriod | SerialNumber | FirmwareVersion | ChickenBits | DeviceType | Scratch0 | Scratch1 | YawFramePeriod | AngularPositionFramePeriod | AngularVelocityFramePeriod | AccelerationFramePeriod | SetYaw | SetPosePositiveW | SetPoseNegativeW | GyroXSensitivity | GyroYSensitivity | GyroZSensitivity | GyroXZroOffset | GyroYZroOffset | GyroZZroOffset | GyroZroOffsetTemperature | TemperatureCalibrationX0 | TemperatureCalibrationY0 | TemperatureCalibrationZ0 | TemperatureCalibrationT0 | TemperatureCalibrationX1 | TemperatureCalibrationY1 | TemperatureCalibrationZ1 | TemperatureCalibrationT1
