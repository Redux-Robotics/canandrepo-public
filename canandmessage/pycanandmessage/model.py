import typing
import dataclasses
from typing import Annotated
import enum
import math
import struct
from . import utils

import can

__all__ = [
    "MessageWrapper",
    "Struct",
    "Bitset",
    "Enum",
    "UInt",
    "SInt",
    "Float",
    "Buffer",
    "Boolean",
    "MessageMeta",
    "SettingMeta",
    "Signal",
    "BaseMessage",
    "BaseSetting",
    "BaseDevice"
]


def rio_hb(
    match_time_seconds : int = 0,
    match_number : int = 0,
    replay_number : int = 0,
    red_alliance : bool = False,
    enabled : bool = False,
    autonomous : bool = False,
    test_mode : bool = False,
    system_watchdog : bool = False,
    tournament_type : int = 0,
    time_of_day_yr : int = 0,
    time_of_day_month : int = 0,
    time_of_day_day : int = 0,
    time_of_day_sec : int = 0,
    time_of_day_min : int = 0,
    time_of_day_hr : int = 0,
) -> can.Message:

    data = (
        ((match_time_seconds & 0xff)) |
        ((match_number & 0x3ff)  << 8) |
        ((replay_number & 0x3f)  << 18) |
        ((red_alliance)          << 24) |
        ((enabled)               << 25) |
        ((autonomous)            << 26) |
        ((test_mode)             << 27) |
        ((system_watchdog)       << 28) |
        ((tournament_type & 0x7) << 29) |
        ((time_of_day_yr & 0x3f) << 32) |
        ((time_of_day_month & 0xf) << 38) |
        ((time_of_day_day & 0x1f) << 42) |
        ((time_of_day_sec & 0x3f) << 47) |
        ((time_of_day_min & 0x3f) << 53) |
        ((time_of_day_hr & 0x1f)  << 59)
    )
    return can.Message(arbitration_id=0x01011840, dlc=8, data=data.to_bytes(8, 'little'))
    pass

class MessageWrapper:
    def __init__(self, data: int, dlc: int, arb_id: int, timestamp: int=0):
        self.timestamp = timestamp
        self.data: int = data
        self.dlc: int = dlc
        self.arb_id: int = arb_id
    
    def as_bytes(self) -> bytes:
        return self.data.to_bytes(self.dlc, "little")[:self.dlc]
    
    def __repr__(self) -> str:
        byte_repr = ", ".join([hex(b) for b in self.as_bytes()])
        return f"MessageWrapper(id = {self.arb_id:x}, data = [{byte_repr}])"
    
    @classmethod
    def from_can(cls, msg: can.Message) -> typing.Self:
        data = int.from_bytes(msg.data, 'little')
        return MessageWrapper(data, msg.dlc, msg.arbitration_id, timestamp=msg.timestamp)
    
    def to_can(self) -> can.Message:
        return can.Message(timestamp=self.timestamp or 0, arbitration_id=self.arb_id, data=self.as_bytes())

class Struct:
    def __init__(self, dtype: typing.Type):
        self.dtype = dtype

class Bitset:
    def __init__[T: enum.Flag](self, width: int, dtype: typing.Type[T], default_value: T):
        self.width = width
        self.dtype = dtype
        self.default_value = default_value

class Enum:
    def __init__[T: enum.IntEnum](self, width: int, dtype: typing.Type[T], default_value: T):
        self.width = width
        self.dtype = dtype
        self.default_value = default_value

@dataclasses.dataclass
class UInt:
    width: int
    min: typing.Optional[int]
    max: typing.Optional[int]
    default_value: int 
    factor_num: int
    factor_den: int
    offset: int

@dataclasses.dataclass
class SInt:
    width: int
    min: typing.Optional[int]
    max: typing.Optional[int]
    default_value: int 
    factor_num: int
    factor_den: int
    offset: int

@dataclasses.dataclass
class Float:
    width: int
    min: typing.Optional[float]
    max: typing.Optional[float]
    default_value: float 
    allow_nan_inf: bool
    factor_num: int
    factor_den: int
    offset: float

@dataclasses.dataclass
class Buffer:
    width: int
    default_value: int # yeah this should _probably_ be bytes

@dataclasses.dataclass
class Boolean:
    default_value: bool # yeah this should _probably_ be bytes

@dataclasses.dataclass
class MessageMeta:
    device_type: int
    id: int
    min_length: int
    max_length: int

@dataclasses.dataclass
class SettingMeta:
    idx: int
    set_setting: typing.Type['BaseMessage']
    report_setting: typing.Type['BaseMessage']
    stg_flags: typing.Type


class Signal:
    def __init__(self, offset: int, meta, optional=False):
        self.offset: int = offset
        self.meta = meta
        self.optional = optional
    
    def decode(self, data: int, max_idx: int):
        if self.offset > max_idx:
            return None
        
        data = data >> self.offset
        meta = self.meta
        match meta:
            case UInt():
                return data & utils.mask(meta.width)
            case SInt():
                value = data & utils.mask(meta.width)
                return (value - (1 << meta.width)) if value >= (1 << (meta.width - 1)) else value
            case Boolean():
                return bool(data & 0b1)
            case Float():
                data = data & utils.mask(meta.width)
                match meta.width:
                    case 24:
                        data = (data & 0xffffff) << 8
                        return struct.unpack("<f", (data << 8).to_bytes(4, 'little'))[0]
                    case 32:
                        return struct.unpack("<f", data.to_bytes(4, 'little'))[0]
                    case 64:
                        return struct.unpack("<d", data.to_bytes(8, 'little'))[0]
                    case _:
                        raise ValueError(f"Float({meta.width}) invalid size!!!")
            case Buffer():
                data = data & utils.mask(meta.width)
                return bytearray(data.to_bytes((meta.width + 7) // 8, 'little'))
            
            case Bitset():
                return data & utils.mask(meta.width)
            
            case Enum():
                return data & utils.mask(meta.width)
            
            case Struct():
                max_idx -= self.offset
                subsig_data = {}
                for subsig_name, subsig_hint in typing.get_type_hints(meta.dtype, include_extras=True).items():
                    if not typing.get_origin(subsig_hint) is typing.Annotated:
                        continue
                    subsig = subsig_hint.__metadata__[0]
                    if not isinstance(subsig, Signal):
                        raise TypeError("signal annotation should be Signal")
                    
                    subsig_data[subsig_name] = subsig.decode(data, max_idx)
                
                return meta.dtype(**subsig_data)

    
    def encode(self, name: str, value) -> int:
        if self.optional and value is None:
            return 0

        meta = self.meta
        ivalue: int = 0
        match meta:
            case UInt():
                value = int(value)
                min_bound = utils.unwrap_or(meta.min, 0)
                max_bound = utils.unwrap_or(meta.max, utils.default_uint_max(meta.width))
                if not (min_bound <= value <= max_bound):
                    raise ValueError(f"{name} out of bounds for {min_bound} <= {value} <= {max_bound}")
                
                ivalue = value & utils.mask(meta.width)

            case SInt():
                value = int(value)
                min_bound = utils.unwrap_or(meta.min, utils.default_sint_min(meta.width))
                max_bound = utils.unwrap_or(meta.max, utils.default_sint_max(meta.width))
                if not (min_bound <= value <= max_bound):
                    raise ValueError(f"{name} out of bounds for {min_bound} <= {value} <= {max_bound}")
                ivalue = value & utils.mask(meta.width)
            case Boolean():
                ivalue = bool(value)
            case Float():
                value = float(value)
                if not (meta.allow_nan_inf or math.isfinite(value)):
                    raise ValueError(f"{name} is non-finite!")
                
                if meta.min is not None and value < meta.min:
                    raise ValueError(f"{name} {value} is less than minimum {meta.min}")
                
                if meta.max is not None and value > meta.min:
                    raise ValueError(f"{name} {value} is greater than maximum {meta.max}")

                match meta.width:
                    case 24:
                        ivalue = int.from_bytes(struct.pack("<f", value), 'little') >> 8
                    case 32:
                        ivalue = int.from_bytes(struct.pack("<f", value), 'little')
                    case 64:
                        ivalue = int.from_bytes(struct.pack("<d", value), 'little')
                    case _:
                        raise ValueError(f"Float({meta.width}) invalid size!!!")
            case Buffer():
                max_len = (meta.width + 7) // 8
                if len(value) > max_len:
                    raise ValueError(f"{name} buffer len {len(value)} > max len {max_len}")

                ivalue = int.from_bytes(value, 'little')

            case Bitset():
                ivalue = value
                
            case Enum():
                ivalue = value

            case Struct():
                ivalue = 0
                for subsig_name, subsig_hint in typing.get_type_hints(meta.dtype, include_extras=True).items():
                    if not typing.get_origin(subsig_hint) is typing.Annotated:
                        continue
                    subsig = subsig_hint.__metadata__[0]
                    if not isinstance(subsig, Signal):
                        raise TypeError("signal annotation should be Signal")
                    subsig_value = getattr(value, subsig_name)
                    ivalue |= subsig.encode(f"{name}.{subsig_name}", subsig_value)

        return ivalue << self.offset

class BaseMessage:
    __meta__: MessageMeta
    def to_wrapper(self, dev_id: int, device_type: int = None) -> MessageWrapper:
        if device_type is None:
            device_type = self.__meta__.device_type

        dlc = self.__meta__.min_length
        data = 0
        for name, hint in typing.get_type_hints(self, include_extras=True).items():

            if not typing.get_origin(hint) is typing.Annotated:
                continue
            sig = hint.__metadata__[0]

            if not isinstance(sig, Signal):
                raise TypeError("signal annotation should be Signal")
            
            value = getattr(self, name)
            if sig.optional: 
                if value is not None:
                    dlc = self.__meta__.max_length
                else:
                    continue
            data |= sig.encode(name, value)
        return MessageWrapper(data, dlc, (device_type << 24) | (0xe << 16) | (self.__meta__.id << 6) | dev_id)
    
    @classmethod
    def from_wrapper(cls, msg: MessageWrapper) -> typing.Optional[typing.Self]:
        data = msg.data
        max_idx = msg.dlc * 8

        subsig_data = {}
        for subsig_name, subsig_hint in typing.get_type_hints(cls, include_extras=True).items():
            if not typing.get_origin(subsig_hint) is typing.Annotated:
                continue
            subsig = subsig_hint.__metadata__[0]
            if not isinstance(subsig, Signal):
                raise TypeError("signal annotation should be Signal")
            subsig_data[subsig_name] = subsig.decode(data, max_idx)
        
        return cls(**subsig_data)

class BaseSetting:
    __meta__: SettingMeta

    def encode(self) -> typing.ByteString:
        data = 0
        for name, hint in typing.get_type_hints(self, include_extras=True).items():
            if not typing.get_origin(hint) is typing.Annotated:
                continue
            sig = hint.__metadata__[0]
            if not isinstance(sig, Signal):
                raise TypeError("signal annotation should be Signal")
            value = getattr(self, name)
            data |= sig.encode(name, value)

        return data.to_bytes(6, 'little')
    
    @classmethod
    def decode(cls, data: typing.ByteString) -> typing.Self:
        data = int.from_bytes(data[:6], 'little')
        sig_data = {}
        for subsig_name, subsig_hint in typing.get_type_hints(cls, include_extras=True).items():
            if not typing.get_origin(subsig_hint) is typing.Annotated:
                continue
            subsig = subsig_hint.__metadata__[0]
            if not isinstance(subsig, Signal):
                raise TypeError("signal annotation should be Signal")
            
            sig_data[subsig_name] = subsig.decode(data, 48)
        
        return cls(**sig_data)
    
    def to_wrapper(self, dev_id: int, ephemeral=False, synch_hold=False, synch_cnt=0) -> MessageWrapper:
        self.__meta__.set_setting(
            address = self.__meta__.idx,
            data = self.encode(),
            flags = self.__meta__.stg_flags(ephemeral=ephemeral, synch_hold=synch_hold, synch_msg_count=synch_cnt)
        ).to_wrapper(dev_id)

class BaseDevice:
    device_type: int
    name: str
    messages: typing.Dict[int, typing.Type[BaseMessage]]
    settings: typing.Dict[int, typing.Type[BaseSetting]]

    @classmethod
    def decode_msg_generic(cls, msg: MessageWrapper) -> BaseMessage | None:
        arb_id = msg.arb_id
        if (arb_id & 0x1fff0000) != ((cls.device_type << 24) | (0xe << 16)):
            return None
        msg_id = (arb_id >> 6) & 0xff
        if msg_id not in cls.messages:
            return None
        return cls.messages[msg_id].from_wrapper(msg)


@dataclasses.dataclass
class TestSettingFlags:
    ephemeral: typing.Annotated[bool, Signal(0, Boolean(False))]
    synch_hold: typing.Annotated[bool, Signal(1, Boolean(False))]
    synch_msg_count: typing.Annotated[int, Signal(4, UInt(4, None, None, 0, 1, 1, 0))]

@dataclasses.dataclass
class TestSetSetting(BaseMessage):
    __meta__ = MessageMeta(0x4, 3, 7, 8)

    address: typing.Annotated[int, Signal(0, UInt(8, 0, 255, 0, 1, 1, 0))]
    data: typing.Annotated[bytearray, Signal(8, Buffer(48, 0))]
    flags: typing.Annotated[TestSettingFlags, Signal(56, Struct(TestSettingFlags), True)]

def gen_test():
    return TestSetSetting(0xe, bytearray([1, 2, 3, 4, 5, 6]), TestSettingFlags(False, True, 3))
