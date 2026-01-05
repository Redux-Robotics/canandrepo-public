import tomllib
from typing import *
import enum
import dataclasses
import pathlib
import struct
from .toml_defs import DeviceSpec, EnumEntrySpec, EnumSpec

__all__ = [
    'UIntMeta',
    'SIntMeta',
    'FloatMeta',
    'BitsetFlag',
    'BufMeta',
    'EnumMeta',
    'EnumEntry',
    'StructMeta',
    'BitsetMeta',
    'PadMeta',
    'BoolMeta',
    'Signal',
    'Source',
    'Message',
    'Setting',
    'Device',
    'DType',
    'parse_spec',
    'parse_spec_to_device'
]

@dataclasses.dataclass
class UIntMeta:
    width: int
    min: Optional[int]
    max: Optional[int]
    default_value: int 
    factor_num: int
    factor_den: int
    offset: int

@dataclasses.dataclass
class SIntMeta:
    width: int
    min: Optional[int]
    max: Optional[int]
    default_value: int 
    factor_num: int
    factor_den: int
    offset: int

@dataclasses.dataclass
class FloatMeta:
    width: int
    min: Optional[float]
    max: Optional[float]
    default_value: float 
    allow_nan_inf: bool
    factor_num: int
    factor_den: int
    offset: float

@dataclasses.dataclass
class BitsetFlag:
    bit_idx: int
    default_value: bool
    name: str
    comment: str

@dataclasses.dataclass
class BufMeta:
    width: int
    default_value: int # yeah this should _probably_ be bytes

@dataclasses.dataclass
class EnumMeta:
    name: str
    width: int
    default_value: str
    default_value_idx: int
    is_public: bool
    values: Dict[int, 'EnumEntry']

@dataclasses.dataclass
class EnumEntry:
    name: str
    comment: str
    index: int

@dataclasses.dataclass
class StructMeta:
    name: str
    signals: List['Signal']

@dataclasses.dataclass
class BitsetMeta:
    name: str
    width: int
    flags: List[BitsetFlag]
    def default_u64(self) -> int:
        v = 0
        for ent in self.flags:
            v |= ent.default_value << ent.bit_idx
        return v

@dataclasses.dataclass
class PadMeta:
    width: int

@dataclasses.dataclass
class BoolMeta:
    default_value: bool

#DType = Union[None, UIntMeta, SIntMeta, BufMeta, FloatMeta, BitsetMeta, PadMeta, BoolMeta, EnumMeta, StructMeta]

@dataclasses.dataclass
class Signal:
    name: str
    comment: str
    dtype: 'DType'
    optional: bool

    @classmethod
    def from_msg(cls, name: str, msg: 'Message') -> Self:
        return cls(
            name=name, 
            comment=msg.comment, 
            dtype=DType(
                StructMeta(
                    name.lower(),
                    msg.signals
                )
            ), optional = False)

    @classmethod
    def from_stg(cls, name: str, stg: 'Setting') -> Self:
        return cls(
            name = name, 
            comment = stg.comment,
            dtype = stg.dtype,
            optional = False,
        )
        pass

class Source(enum.StrEnum):
    Device = "Device"
    Host = "Host"
    Both = "Both"

    def from_str(s: str) -> Self:
        return {
            "device": Source.Device,
            "host": Source.Host,
            "bidir": Source.Both,
            "both": Source.Both
        }[s]

@dataclasses.dataclass
class Message:
    id: int
    comment: str
    max_length: int
    min_length: int
    source: Source
    is_public: bool
    signals: List[Signal]

@dataclasses.dataclass
class Setting:
    name: str
    id: int
    comment: str
    dtype: 'DType'
    readable: bool
    writable: bool
    reset_on_default: bool
    vendordep: bool
    vdep_setting: bool
    special_flags: List[str]

@dataclasses.dataclass
class Device:
    name: str
    arch: str
    dev_type: int
    dev_class: int
    java_package: str
    cpp_namespace: str
    messages: Dict[str, Message]
    settings: Dict[str, Setting]
    enums: Dict[str, EnumMeta]
    structs: Dict[str, StructMeta]
    bitsets: Dict[str, BitsetMeta]
    spec: DeviceSpec

DTypeOnion = Union[None, UIntMeta, SIntMeta, BufMeta, FloatMeta, BitsetMeta, PadMeta, BoolMeta, EnumMeta, StructMeta]
class DType:
    def __init__(self, meta: DTypeOnion):
        self.meta = meta
    
    def bit_length(self):
        match self.meta:
            case None:
                return 0
            case BoolMeta():
                return 1
            case StructMeta():
                return sum(sig.dtype.bit_length() for sig in self.meta.signals)
            case other:
                return self.meta.width
    
    def is_pad(self):
        return self.meta is None or isinstance(self.meta, PadMeta)
    
    def canonical_name(self):
        match self.meta:
            case UIntMeta():
                return f"uint:{self.meta.width}"
            case SIntMeta():
                return f"sint:{self.meta.width}"
            case FloatMeta():
                return f"float:{self.meta.width}"
            case BoolMeta():
                return "bool"
            case PadMeta():
                return f"pad:{self.meta.width}"
            case StructMeta():
                return f"struct:{self.meta.name}"
            case BitsetMeta():
                return f"bitset:{self.meta.width}"
            case BufMeta():
                return f"buf:{self.meta.width}"
            case EnumMeta():
                return f"enum:{self.meta.name}"
            case aaa:
                raise ValueError(f"DType::None encountered: {aaa}")

    def default_value_as_bits(self) -> int:
        meta = self.meta
        match meta:
            case UIntMeta():
                return meta.default_value
            case SIntMeta():
                return meta.default_value
            case FloatMeta():
                match meta.width:
                    case 24:
                        return int.from_bytes(struct.pack("<f", meta.default_value), 'little') >> 8
                    case 32:
                        return int.from_bytes(struct.pack("<f", meta.default_value), 'little')
                    case 64:
                        return int.from_bytes(struct.pack("<d", meta.default_value), 'little')
                    case _:
                        raise ValueError(f"Float({meta.width}) invalid size!!!")
            case BoolMeta():
                return int(meta.default_value)
            case PadMeta():
                return 0
            case StructMeta():
                ivalue = 0
                ishift = 0
                for subsig in meta.signals:
                    ivalue |= subsig.dtype.default_value_as_bits() << ishift
                    ishift += subsig.dtype.bit_length()
                return ivalue
            case BitsetMeta():
                return meta.default_u64()
            case BufMeta():
                return meta.default_value
            case EnumMeta():
                if meta.default_value:
                    return meta.default_value_idx
                else:
                    return 0
            case _:
                return None

def parse_spec(spec_path: pathlib.Path) -> DeviceSpec:
    if isinstance(spec_path, str):
        spec_path = pathlib.Path(spec_path)
    with open(spec_path, "rb") as f:
        dev_spec_data = tomllib.load(f)
    
    dev_spec = DeviceSpec.from_dict(dev_spec_data)
    dev: DeviceSpec
    upper_dev: DeviceSpec = dev_spec
    for base in dev_spec.base:
        with open(spec_path.parent/f"{base.lower()}.toml", "rb") as f:
            base_spec = DeviceSpec.from_dict(tomllib.load(f))
        base_spec.arch = upper_dev.arch
        for base_dev_name in upper_dev.base:
            if base_dev_name not in base_spec.base:
                base_spec.base.append(base_dev_name)
        base_spec.dev_class = upper_dev.dev_class
        base_spec.dev_type = upper_dev.dev_type
        base_spec.name = upper_dev.name

        base_spec.enums.update(upper_dev.enums)
        base_spec.types.update(upper_dev.types)
        base_spec.msg.update(upper_dev.msg)
        base_spec.settings.update(upper_dev.settings)
        base_spec.setting_commands.update(upper_dev.setting_commands)
        base_spec.vendordep = upper_dev.vendordep
        upper_dev = base_spec
    dev = upper_dev
    dev.enums['SETTING'] = EnumSpec.from_dict({
        "btype": "uint",
        "bits": 8,
        "is_public": True,
        "default_value": "",
        "values": {name: {"id": stg.id, "comment": stg.comment} for name, stg in dev.settings.items()}
    })
    dev.enums['SETTING_COMMAND'] = EnumSpec.from_dict({
        "btype": "uint",
        "bits": 8,
        "is_public": True,
        "default_value": "",
        "values": {name: {"id": cmd.id, "comment": cmd.comment} for name, cmd in dev.setting_commands.items()}
    })

    return dev 

def parse_spec_to_device(spec_path: pathlib.Path) -> Device:
    from .model_impl import impl_Device_from
    return impl_Device_from(parse_spec(spec_path))