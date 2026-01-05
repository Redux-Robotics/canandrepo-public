from . import *
from . import toml_defs
from .utils import *
from typing import *

def impl_DType_from_type(type_name: str, type_def: toml_defs.TypeSpec, default_value: Any, dev: toml_defs.DeviceSpec):
    if default_value is None:
        default_value = type_def.default_value
    width = type_def.bits
    match type_def.btype:
        case "uint":
            return DType(UIntMeta(
                width = width,
                min = int(unwrap_or(type_def.min, 0)),
                max = int(unwrap_or(type_def.max, default_uint_max(width))),
                default_value = unwrap_or(default_value, 0),
                factor_num = type_def.factor[0],
                factor_den = type_def.factor[1],
                offset = 0))
        case "sint":
            return DType(SIntMeta(
                width = width,
                min = int(unwrap_or(type_def.min, default_sint_min(width))),
                max = int(unwrap_or(type_def.max, default_sint_max(width))),
                default_value = unwrap_or(default_value, 0),
                factor_num = type_def.factor[0],
                factor_den = type_def.factor[1],
                offset = 0))
        case "buf":
            return DType(BufMeta(
                width = width,
                default_value = unwrap_or(default_value, 0)))
        case "float":
            return DType(FloatMeta(
                width = width,
                min = none_map(type_def.min, float),
                max = none_map(type_def.max, float),
                default_value = unwrap_or(none_map(default_value, float), 0),
                allow_nan_inf = type_def.allow_nan_inf,
                factor_num = type_def.factor[0],
                factor_den = type_def.factor[1],
                offset = 0))
        case "bitset":
            return DType(impl_BitsetMeta_from(type_name, type_def))
        case "pad":
            return DType(PadMeta(width))
        case "bool":
            return DType(BoolMeta(default_value=default_value))
        case "struct":
            return DType(impl_StructMeta_from(type_name, type_def, dev))
        case _:
            return impl_DType_from_type(type_def.btype, dev.types[type_def.btype], default_value, dev)
            pass

def impl_DType_from_sig(dev: toml_defs.DeviceSpec, dtype_name: str, default_value: Any) -> DType:
    nsplit = dtype_name.split(":")
    match nsplit[0]:
        case "none":
            return DType(None)
        case "buf":
            width = int(nsplit[1])
            return DType(BufMeta(
                width = width,
                default_value = unwrap_or(none_map(default_value, int), 0)
            ))
        case "uint":
            width = int(nsplit[1])
            return DType(UIntMeta(
                width = width,
                min = 0,
                max = default_uint_max(width),
                default_value = unwrap_or(none_map(default_value, int), 0),
                factor_num = 1,
                factor_den = 1,
                offset = 0,
            ))
        case "sint":
            width = int(nsplit[1])
            return DType(SIntMeta(
                width = width,
                min = default_sint_min(width),
                max = default_sint_max(width),
                default_value = unwrap_or(none_map(default_value, int), 0),
                factor_num = 1,
                factor_den = 1,
                offset = 0,
            ))
        case "float":
            width = int(nsplit[1])
            return DType(FloatMeta(
                width = width,
                min = None,
                max = None,
                default_value = unwrap_or(none_map(default_value, float), 0),
                allow_nan_inf = True,
                factor_num = 1,
                factor_den = 1,
                offset = 0,
            ))
        case "pad":
            width = int(nsplit[1])
            return DType(PadMeta(width))
        case "bool":
            return DType(BoolMeta(bool(default_value)))
        case "setting_data":
            return DType(BufMeta(width = 48, default_value = 0))
        case "enum":
            return DType(impl_EnumMeta_from(nsplit[1], dev.enums[nsplit[1]], default_value))
        case _:
            return impl_DType_from_type(dtype_name, dev.types[dtype_name], default_value, dev)

def impl_Signal_from(sgnl: toml_defs.MessageSignalSpec, dev: toml_defs.DeviceSpec) -> Signal:
    return Signal(
        name = sgnl.name,
        comment = sgnl.comment,
        dtype = impl_DType_from_sig(dev, sgnl.dtype, sgnl.default_value),
        optional = sgnl.optional
    )

def impl_Signal_from_Setting(value: Setting) -> Signal:
    return Signal(
        name = "value", 
        comment = "setting value",
        dtype = value.dtype,
        optional = False,
    )

# impl Source.flip and Source.from is on Source

def impl_Message_from(dm: toml_defs.DeviceMessageSpec, dev: toml_defs.DeviceSpec) -> Message:
    if dm.length is not None:
        min_length, max_length = (dm.length, dm.length)
    else:
        min_length, max_length = (unwrap_or(dm.min_length, 0), unwrap_or(dm.max_length, 8))
    
    return Message(
        id = dm.id, 
        min_length = min_length,
        max_length = max_length,
        comment = dm.comment,
        is_public = dm.is_public,
        signals = [impl_Signal_from(v, dev) for v in dm.signals],
        source = Source.from_str(dm.source)
    )


def impl_Setting_from(name: str, value: toml_defs.DeviceSettingSpec, dev: toml_defs.DeviceSpec) -> Setting:
    dtype = impl_DType_from_sig(dev, value.dtype, value.default_value)
    print(value)

    return Setting(
        name = name,
        id = value.id, 
        comment = value.comment,
        dtype = dtype,
        readable = value.readable,
        writable = value.writable,
        reset_on_default = value.reset_on_default,
        vendordep = value.vendordep,
        vdep_setting = value.vdep_setting,
        special_flags = list(value.special_flags)
    )

def impl_EnumMeta_from(name: str, entry: toml_defs.EnumSpec, default_value: Optional[str]) -> EnumMeta:
    default_value = default_value or entry.default_value
    default_value_idx = 0
    if default_value not in entry.values:
        if name in ("SETTING", "SETTING_COMMAND"):
            default_value = ""
        else:
            panic(ValueError(f"invalid enum default value {name}::{default_value}."))
    else:
        default_value_idx = entry.values[default_value].id
    

    return EnumMeta(
        name = name,
        width = entry.bits,
        default_value = default_value,
        default_value_idx = default_value_idx,
        is_public = entry.is_public,
        values = {
            ent.id: EnumEntry(
                name = ent_name,
                comment = ent.comment,
                index = ent.id) 
            for ent_name, ent in entry.values.items()}
    )

def impl_BitsetMeta_from(name: str, type_def: toml_defs.TypeSpec) -> BitsetMeta:
    default_value = unwrap_or(none_map(type_def.default_value, int), 0)
    return BitsetMeta(
        name = name,
        width = type_def.bits,
        flags = [
            BitsetFlag(
                bit_idx = i,
                default_value = ((default_value >> i) & 1) > 0,
                name = x.name,
                comment = x.comment) 
            for i, x in enumerate(type_def.bit_flags)])

def impl_StructMeta_from(name: str, ent: toml_defs.TypeSpec, dev: toml_defs.DeviceSpec) -> StructMeta:
    return StructMeta(
        name = name,
        signals = [
            Signal(
                name = sig.name,
                comment = sig.comment,
                dtype = impl_DType_from_sig(dev, sig.dtype, sig.default_value),
                optional = sig.optional) for sig in ent.signals])

def impl_Device_from(dev_spec: toml_defs.DeviceSpec) -> Device:
    return Device(
        name = dev_spec.name,
        arch = dev_spec.arch,
        dev_type = dev_spec.dev_type,
        dev_class = dev_spec.dev_class,
        messages = { name: impl_Message_from(msg, dev_spec) for name, msg in dev_spec.msg.items() },
        settings = { name: impl_Setting_from(name, stg, dev_spec) for name, stg in dev_spec.settings.items() },
        enums = { name: impl_EnumMeta_from(name, ent, None) for name, ent in dev_spec.enums.items() },
        bitsets = { name: impl_BitsetMeta_from(name, ent) for name, ent in dev_spec.types.items() if ent.btype == "bitset"},
        structs = { name: impl_StructMeta_from(name, ent, dev_spec) for name, ent in dev_spec.types.items() if ent.btype == "struct"}, 
        java_package = f"{dev_spec.vendordep.java_package}" if dev_spec.vendordep else "",
        cpp_namespace = f"{dev_spec.vendordep.cpp_namespace}" if dev_spec.vendordep else "",
        spec = dev_spec
    )