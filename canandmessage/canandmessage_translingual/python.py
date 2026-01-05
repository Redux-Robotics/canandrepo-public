import typing
import textwrap
import enum
import math
from pathlib import Path

from .canandmessage_parser import *
from .canandmessage_parser import utils

def doc_comment(s: str) -> str:
    return f'"""{s}"""'


template = """
import enum

"""

enum_template = """class {name}(enum.IntEnum):
{entries}
"""

bitset_template = """class {name}(enum.Flag):
{entries}
"""

struct_template = """
@dataclasses.dataclass
class {name}:
{entries}
"""

def indent4(s: str) -> str:
    return textwrap.indent(s, "    ")

def gen_enumers(dev: Device) -> str:
    enumers = []
    for name, enum_meta in dev.enums.items():
        entries = []
        for _, ent in enum_meta.values.items():
            entries.append(f"    {ent.name} = 0x{ent.index:x}")
            entries.append(indent4(doc_comment(ent.comment)))
        enumers.append(enum_template.format(
            name = utils.screaming_snake_to_camel(name),
            entries = "\n".join(entries)
        ))
    
    return "\n".join(enumers)

def gen_bitsets(dev: Device) -> str:
    bitsets = []
    for name, bitset_meta in dev.bitsets.items():
        entries = []
        for ent in bitset_meta.flags:
            entries.append(f"    {ent.name.upper()} = 0x{1 << ent.bit_idx:x}")
            entries.append(indent4(doc_comment(ent.comment)))
        bitsets.append(bitset_template.format(
            name = utils.screaming_snake_to_camel(name),
            entries = "\n".join(entries) or "    pass"
        ))
    
    return "\n".join(bitsets)

def gen_composite_signal(signals: typing.List[Signal], prefix="") -> str:
    idx = 0
    entries = []
    for ent in signals:
        dtype_name = name_for_dtype(ent.dtype, prefix=prefix)
        if dtype_name is None:
            idx += ent.dtype.bit_length()
            continue
        active_sig_template = sig_template
        if ent.optional:
            dtype_name = f"Optional[{dtype_name}]"
            active_sig_template = sig_template_optional
        entries.append(active_sig_template.format(
            name = ent.name,
            htype = dtype_name,
            offset = idx,
            dtype = meta_for_dtype(ent.dtype, prefix=prefix)
        ))
        entries.append(indent4(doc_comment(ent.comment)))
        idx += ent.dtype.bit_length()
    
    return "\n".join(entries)


def gen_structs(dev: Device) -> str:
    structs = []
    for name, struct_meta in dev.structs.items():
        entries = gen_composite_signal(struct_meta.signals)
        structs.append(struct_template.format(
            name = utils.screaming_snake_to_camel(name),
            entries = entries,
        ))
    
    return "\n".join(structs)


def name_for_dtype(dtype: DType, prefix="") -> str | None:
    match dtype.meta:
        case UIntMeta():
            return "int"
        case SIntMeta():
            return "int"
        case FloatMeta():
            return "float"
        case BoolMeta():
            return "bool"
        case PadMeta():
            return None
        case StructMeta():
            return prefix + utils.screaming_snake_to_camel(dtype.meta.name)
        case BitsetMeta():
            return prefix + utils.screaming_snake_to_camel(dtype.meta.name)
        case BufMeta():
            return "bytearray"
        case EnumMeta():
            return prefix + utils.screaming_snake_to_camel(dtype.meta.name)
        case _:
            return None

def meta_for_dtype(dtype: DType, prefix="") -> str | None:
    meta = dtype.meta
    match meta:
        case UIntMeta():
            return (f"UInt(width={meta.width}, min={meta.min}, max={meta.max}, default_value={meta.default_value}, "
                    f"factor_num={meta.factor_num}, factor_den={meta.factor_den}, offset={meta.offset})")
        case SIntMeta():
            return (f"SInt(width={meta.width}, min={meta.min}, max={meta.max}, default_value={meta.default_value}, "
                    f"factor_num={meta.factor_num}, factor_den={meta.factor_den}, offset={meta.offset})")
        case FloatMeta():
            default_value = meta.default_value
            if not math.isfinite(default_value):
                if math.isnan(default_value):
                    default_value = "math.nan"
                elif math.isinf(default_value):
                    if default_value < 0.0:
                        default_value = "-math.inf"
                    else:
                        default_value = "math.inf"

            return (f"Float(width={meta.width}, min={meta.min}, max={meta.max}, default_value={default_value}, "
                    f"allow_nan_inf={meta.allow_nan_inf}, factor_num={meta.factor_num}, factor_den={meta.factor_den}, offset={meta.offset})")
        case BoolMeta():
            return f"Boolean({meta.default_value})"
        case PadMeta():
            return None
        case StructMeta():
            return f"Struct({prefix + utils.screaming_snake_to_camel(dtype.meta.name)})"
        case BitsetMeta():
            return f"Bitset(width={meta.width}, dtype={prefix + utils.screaming_snake_to_camel(dtype.meta.name)}, default_value={meta.default_u64()})"
        case BufMeta():
            return f"Buffer(width={meta.width}, default_value={meta.default_value.to_bytes((meta.width + 7) // 8, 'little')})"
        case EnumMeta():
            dtype = prefix + utils.screaming_snake_to_camel(dtype.meta.name)
            if meta.default_value:
                default_value = f"{dtype}.{meta.default_value}"
            else:
                default_value = 0

            return f"Enum(width={meta.width}, dtype={dtype}, default_value={default_value})"
        case _:
            return None

def gen_types(dev: Device) -> str:
    return f"""import enum
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *

{gen_enumers(dev)}
{gen_bitsets(dev)}
{gen_structs(dev)}
"""
#import .types as device_types

sig_template = """    {name}: Annotated[{htype}, Signal({offset}, {dtype})]"""
sig_template_optional = """    {name}: Annotated[{htype}, Signal({offset}, {dtype}, optional=True)]"""

msg_template = """
@dataclasses.dataclass
class {name}(BaseMessage):
{comment}
    __meta__ = MessageMeta(device_type={device_type}, id={id}, min_length={min_length}, max_length={max_length})
{entries}

"""

msg_header = """
import dataclasses
from typing import Optional, Annotated
from . import types as device_types
from pycanandmessage.model import *
"""

def gen_msg(dev: Device) -> str:
    variants = [msg_header]
    names = []
    for name, msg in dev.messages.items():
        entries = gen_composite_signal(msg.signals, prefix="device_types.")
        camel_name = utils.screaming_snake_to_camel(name)
        names.append(camel_name)
        variants.append(msg_template.format(
            name = camel_name,
            comment = f"    {doc_comment(msg.comment)}",
            entries = entries,

            device_type = dev.dev_type, 
            id = msg.id,
            min_length = msg.min_length,
            max_length = msg.max_length,
        ))
    
    variants.append("__all__ = ['MessageType', " + ", ".join(map(repr, names)) + "]")
    
    return "\n".join(variants) + "\n\ntype MessageType = " + " | ".join(names)


stg_header = """
import dataclasses
import math
from typing import Optional, Annotated
from . import types as device_types
from . import message
from pycanandmessage.model import *

__all__ = ['SettingType', {names}]

{stgs}

type SettingType = {names_or}
"""

stg_template = """
@dataclasses.dataclass
class {name}(BaseSetting):
{comment}
    __meta__ = SettingMeta(idx=0x{idx:x}, set_setting=message.SetSetting, report_setting=message.ReportSetting, stg_flags=device_types.SettingFlags)
{entries}
"""

def gen_stg(dev: Device) -> str:
    stgs = []
    names = []
    for name, stg in dev.settings.items():
        entries = gen_composite_signal([Signal("value", "Setting value", stg.dtype, False)], prefix="device_types.")
        camel_name = utils.screaming_snake_to_camel(name)
        names.append(camel_name)
        stgs.append(stg_template.format(
            name = camel_name,
            comment = f"    {doc_comment(stg.comment)}",
            entries = entries,
            idx = stg.id,
        ))
    
    return stg_header.format(names=", ".join(map(repr, names)), stgs="\n".join(stgs), names_or = " | ".join(names))

    pass

def mask(s: int) -> int:
    return (1 << s) - 1

def gen_valueerror(msg) -> str:
    return f"raise ValueError(f'{msg}')"


device_template = """
from pycanandmessage import BaseDevice, MessageWrapper
from . import types, message as msg, setting as stg

class {cname}(BaseDevice):
    device_type = {dev_type}
    msg = msg
    stg = stg
    types = types

    name = {cname!r}
    messages = {msg_map}

    @classmethod
    def decode_msg(cls, msg: MessageWrapper) -> msg.MessageType | None:
        return cls.decode_msg_generic(msg)
"""

def gen_device(dev: Device, pkg_root: Path):
    dev_dir = Path(pkg_root)/dev.name.lower()
    dev_dir.mkdir(parents=True, exist_ok=True)
    # device/__init__.py

    with open(dev_dir/"__init__.py", "w") as f:
        f.write(device_template.format(
            dev_type = dev.dev_type,
            cname = dev.name.capitalize(), 
            msg_map = "{\n" + textwrap.indent(
                "\n".join(
                    f'{ent.id}: msg.{utils.screaming_snake_to_camel(name)},' 
                    for name, ent in dev.messages.items()), " " * 8) + "\n    }"))
    
    # device/types.py
    with open(dev_dir/"types.py", "w") as f:
        f.write(gen_types(dev))
    
    # device/message.py
    with open(dev_dir/"message.py", "w") as f:
        f.write(gen_msg(dev))
    
    # device/setting.py
    with open(dev_dir/"setting.py", "w") as f:
        f.write(gen_stg(dev))

if __name__ == "__main__":
    import sys
    path = Path(sys.argv[1])
    for toml_file in path.glob("*.toml"):
        gen_device(parse_spec_to_device(toml_file), "pycanandmessage")