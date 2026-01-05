from typing import Type, Dict, Tuple, List
from pathlib import Path
from enum import Enum
import jinja2
import os
import csv

from canandmessage_translingual.canandmessage_parser.model_impl import impl_DType_from_sig, impl_DType_from_type
from .canandmessage_parser import *
from .canandmessage_parser import toml_defs
from .canandmessage_parser import DTypeOnion

env = jinja2.Environment(loader=jinja2.FileSystemLoader(Path(__file__).parent))

def header(s, c="=") -> str:
    """Generates a restructured text header"""
    return f"{s}\n{c*len(s)}\n"

def table(headers, body, enable_header=True) -> str:
    headers = [*map(str, headers)]
    body = [[*map(str, line)] for line in body]
    max_lens = [len(h) + 8 for h in headers]
    for line in body:
        for i in range(len(max_lens)):
            max_lens[i] = max(max_lens[i], len(line[i]))
    
    edge_border = "  ".join(["=" * m for m in max_lens])
    title = "  ".join([f"**{h}**".ljust(l) for h, l in zip(headers, max_lens)])
    if enable_header:
        buf = f"\n{edge_border}\n{title}\n{edge_border}\n"
    else:
        buf = f"\n{edge_border}\n"
    for line in body:
        buf +=  "  ".join([v.ljust(l) for v, l in zip(line, max_lens)]) + "\n"
    
    return buf + edge_border + "\n"

def csv2table(csv_str: str):
    reader = csv.reader(csv_str.splitlines())
    header: List[str] = next(reader)
    rest: List[List[str]] = list(reader)
    column_lengths = [len(s) + 4 for s in header]
    for line in rest:
        for col in range(len(line)):
            column_lengths[col] = max(column_lengths[col], len(line[col]))

    horiz_border =  "+" + "+".join(["-" * (n + 2) for n in column_lengths]) + "+"

    # this makes a format string. this is safe because column_lengths is guarenteed to be int
    line_fmt = "|" + "|".join(" {: <" + str(c) + "} " for c in column_lengths) + "|"

    buf = [horiz_border, line_fmt.format(*header), horiz_border.replace("-", "=")]
    buf.extend(line_fmt.format(*line) + "\n" + horiz_border for line in rest)

    return "\n".join(buf)

class ReadState(Enum):
    ENTRY = 0
    COLLECT = 1

def read_external_rst(rst_root: Path, dev: toml_defs.DeviceSpec) -> Dict[str, str]:

    info: Dict[str, str] = {}
    for rst_name in list(dev.base) + [dev.name.lower()]:
        with open(rst_root/f"{rst_name.lower()}.rst") as f:
            rst = f.read()
        
        key = None
        collect = []

        for line in rst.splitlines():
            line = line.rstrip()
            if line.startswith(".. _") and line.endswith(":"):
                if key is not None:
                    info[key] = "\n".join(collect)
                key = line[4:-1]
                collect = []
            else:
                collect.append(line)
        if key is not None:
            info[key] = "\n".join(collect)
    
    # let's do some post-processing for frame periods
    for name, msg in dev.msg.items():
        if msg.frame_period_setting is not None:
            key = f"msg_{name.lower()}"
            info[key] = (info.get(key, "") +
                f"\nThe period at which this message is broadcasted at "
                f"is controlled by the :ref:`{msg.frame_period_setting}"
                f"<setting_{msg.frame_period_setting.lower()}>` setting.\n")

            key = f"setting_{msg.frame_period_setting.lower()}"
            if not info.get(key, "").strip():
                info[key] = (
                    f"Period between each transmission of :ref:`{name}<msg_{name.lower()}>` messages.\n\n"
                    "A value of 0 disables transmission of the associated message altogether."
                )
    return info
            


def renderable_messages(dev: toml_defs.DeviceSpec) -> List[Tuple[str, toml_defs.DeviceMessageSpec]]:
    return sorted([(name, msg) for name, msg in dev.msg.items() if msg.is_public], key=lambda x: -x[1].id)

def renderable_settings(dev: toml_defs.DeviceSpec) -> List[Tuple[str, toml_defs.DeviceSettingSpec]]:
    return sorted([(name, stg) for name, stg in dev.settings.items() if stg.is_public], key=lambda x: -x[1].id)

def renderable_setting_commands(dev: toml_defs.DeviceSpec) -> List[Tuple[str, toml_defs.SettingCommandSpec]]:
    return sorted([(name, cmd) for name, cmd in dev.setting_commands.items()], key=lambda x: x[1].id)

def render_meta_table(dev: toml_defs.DeviceSpec):
    return table(["prop", "val"], [
        ["FRC CAN Device Type", hex(dev.dev_type)],
        ["DBC File", f"`Link <../_static/dbc/{dev.name.lower()}.dbc>`_"],
        ["Inherits from", "[" + ", ".join([f":doc:`{base}`" for base in dev.base])+ "]"],
    ], False)

def render_msg_summary_table(dev: toml_defs.DeviceSpec):
    tbl = []
    for name, msg in renderable_messages(dev):
        tbl.append([f"{hex(msg.id)}", f":ref:`{name}<{dev.name.lower()}_msg_{name.lower()}>`", msg.comment]) 
    return table(["API Index", "Message", "Description"], tbl)

def render_msg_table(msg: toml_defs.DeviceMessageSpec, dev: toml_defs.DeviceSpec):
    tbl = [["API Index", hex(msg.id)]]

    (min_length, max_length) = (msg.min_length, msg.max_length)

    if msg.min_length is not None or msg.max_length is not None:
        min_length = msg.min_length or 0
        max_length = msg.max_length or 8
    if min_length != max_length:
        tbl.append(["Minimum message length", f"{min_length} bytes"]) 
        tbl.append(["Maximum message length", f"{max_length} bytes"]) 
    else:
        if msg.length is not None:
            tbl.append(["Message length", f"{msg.length} bytes"]) 
        else:
            tbl.append(["Message length", f"{msg.min_length} bytes"]) 
    
    tbl.append(["Transmission direction", {
        Source.Device: "Device -\\> robot",
        Source.Host: "Robot -\\> device",
        Source.Both: "Bidirectional",
    }[Source.from_str(msg.source)]])

    if msg.frame_period_setting is not None:
        stg = dev.settings[msg.frame_period_setting]
        tbl.append(["Frame period setting", f":ref:`{msg.frame_period_setting}<{dev.name.lower()}_setting_{msg.frame_period_setting.lower()}>`"])
        tbl.append(["Default frame period", f"{render_stg_default_value(dev, stg)} milliseconds"])


    return table(["Property", "Value"], tbl)

def render_msg_signal_table(signals: List[toml_defs.MessageSignalSpec]):
    rows = []
    for sig in signals:
        rows.append([f"``{sig.name}``", format_dtype(sig.dtype), render_bool_as_check(sig.optional), sig.comment]) 

    return table(['Signal name', 'Signal type', 'Optional', 'Description'], rows)

def render_bool_as_check(value: bool) -> str:
    if value:
        return "✅"
    return "❌"

def render_buf_int(value: int, width: int) -> str:
    ibytes = value.to_bytes((width + 1) // 8, 'little')

    return f"``[{', '.join(hex(x) for x in ibytes)}]``"

def render_stg_default_value(dev: toml_defs.DeviceSpec, stg: toml_defs.DeviceSettingSpec) -> str:
    dtype = impl_DType_from_sig(dev, stg.dtype, stg.default_value)
    
    dtype_str = stg.dtype
    if not (stg.readable and stg.writable):
        return "n/a"

    return render_default_value(dev, dtype_str, stg.default_value, dtype)


def render_default_value(dev: toml_defs.DeviceSpec, dtype: str, default_value, dtype_obj: DType = None) -> str:
    if default_value is None:
        if dtype.startswith("enum:"):
            if dtype_obj is not None:
                meta: EnumMeta = dtype_obj.meta
                return meta.default_value

            return f":ref:`Enum default<{dev.name.lower()}_enum_{dtype[5:].lower()}>`"
        elif (dtype.startswith("sint:") or 
            dtype.startswith("uint:") or
            dtype.startswith("pad:")):
            return "``0``"
        elif dtype.startswith("bool"):
            return "``false``"
        elif dtype.startswith("float"):
            return "``0.0``"
        elif (dtype.startswith("buf:")):
            return render_buf_int(0, int(dtype[4:]))
        else:
            if dtype_obj is not None:
                meta: DTypeOnion = dtype_obj.meta
                match meta:
                    case UIntMeta() | SIntMeta() | FloatMeta() | EnumMeta():
                        return f"``{str(meta.default_value)}``"
                    case BoolMeta():
                        return f"``{str(meta.default_value).lower()}``"
                    case BufMeta():
                        return render_buf_int(meta.default_value, meta.width)
                    case PadMeta():
                        return "``0``"
                    case _:
                        pass
            return f":ref:`Type default<{dev.name.lower()}_type_{dtype.lower()}>`"
    else:
        if dtype.startswith("buf:") and isinstance(default_value, int):
            return render_buf_int(default_value, int(dtype[4:]))
        return str(default_value)


def render_stg_summary_table(dev: toml_defs.DeviceSpec) -> str:
    tbl = []
    for name, stg in renderable_settings(dev):
        tbl.append([
            f"{hex(stg.id)}", # setting index
            f":ref:`{name}<{dev.name.lower()}_setting_{name.lower()}>`", # name
            format_dtype(stg.dtype), # type
            render_stg_default_value(dev, stg), # default value
            render_bool_as_check(stg.readable), # readable
            render_bool_as_check(stg.writable), # writable
            render_bool_as_check(stg.reset_on_default and stg.readable and stg.writable), # reset on default
            stg.comment,
        ]) 
    return table(["Setting index", "Name", "Type", "Default value", "Readable", "Writable", "Resets to factory default", "Description"], tbl)

def format_dtype(dtype: str) -> str:
    if dtype.startswith("enum:"):
        if dtype == "enum:SETTING":
            return f":ref:`Setting index<{dev.name.lower()}_enum_setting>`"

        return f":ref:`{dtype[5:]}<{dev.name.lower()}_enum_{dtype[5:].lower()}>`"
    if dtype.startswith("sint:"):
        return f"``int{dtype[5:]}_t``"
    if dtype.startswith("uint:"):
        return f"``uint{dtype[5:]}_t``"
    if dtype.startswith("buf:"):
        n_bytes = (int(dtype[4:]) + 1) // 8
        return f"``uint8_t[{n_bytes}]``"
    if dtype.startswith("pad:"):
        return f"``pad{dtype[4:]}_t``"
    if dtype == "bool" or dtype == "bit":
        return "``bool``"
    if dtype.startswith("float:"):
        width = dtype[-2:]
        if width == "24":
            return "``float24_t``"
        if width == "32":
            return "``float32_t``"
        else:
            return "``double``"
    return f":ref:`{dtype}<{dev.name.lower()}_type_{dtype.lower()}>`"

def render_setting_table(dev: toml_defs.DeviceSpec, stg: toml_defs.DeviceSettingSpec):
    return table(["Property", "Value"], [
        ["Setting index", hex(stg.id)],
        ['Type', format_dtype(stg.dtype)],
        ["Default value", render_stg_default_value(dev, stg)],
        ["Readable", render_bool_as_check(stg.readable)],
        ["Writable", render_bool_as_check(stg.writable)],
        ["Resets on factory default", render_bool_as_check(stg.reset_on_default and stg.readable and stg.writable)],
    ])

def render_stgcmd_summary_table(dev: toml_defs.DeviceSpec) -> str:
    tbl = []
    for name, cmd in renderable_setting_commands(dev):
        tbl.append([
            f"{hex(cmd.id)}", # Index
            f":ref:`{name}<{dev.name.lower()}_stgcmd_{name.lower()}>`", # name
            cmd.comment, # comment
        ]) 
    return table(["Setting command index", "Name", "Description"], tbl)

def render_setting_cmd_table(stg_cmd: toml_defs.SettingCommandSpec):
    return table(["Property", "Value"], [['Setting command index', hex(stg_cmd.id)]])

def render_type(name: str, type_spec: toml_defs.TypeSpec, dev: toml_defs.DeviceSpec):
    dtype = impl_DType_from_type(name, type_spec, None, dev)
    tbl = []
    meta = dtype.meta
    match meta:
        case UIntMeta():
            tbl.extend([
                ["Base type", type_spec.btype],
                ["Bit width", type_spec.bits],
                ["Minimum value", meta.min],
                ["Maximum value", meta.max],
                ["Default value", meta.default_value],
            ])
            if type_spec.unit:
                tbl.append(["Conversion factor", f"1 LSB = :math:`\\frac{{{type_spec.factor[0]}}}{{{type_spec.factor[1]}}}` {type_spec.unit}"])
            return table(["Property", "Value"], tbl)
        case SIntMeta():
            tbl.extend([
                ["Base type", type_spec.btype],
                ["Bit width", type_spec.bits],
                ["Minimum value", meta.min],
                ["Maximum value", meta.max],
                ["Default value", meta.default_value],
            ])
            if type_spec.unit:
                tbl.append(["Conversion factor", f"1 LSB = :math:`\\frac{{{type_spec.factor[0]}}}{{{type_spec.factor[1]}}}` {type_spec.unit}"])
                #tbl.append(["Conversion factor", f"1 LSB = {type_spec.factor[0]} / {type_spec.factor[1]} {type_spec.unit}"])
            return table(["Property", "Value"], tbl)
        case FloatMeta():
            tbl.extend([
                ["Base type", type_spec.btype],
                ["Bit width", type_spec.bits],
                ["Minimum value", meta.min if meta.min is not None else "n/a"],
                ["Maximum value", meta.max if meta.max is not None else "n/a"],
                ["Default value", meta.default_value],
            ])
            if type_spec.unit:
                #tbl.append(["Conversion factor", f"1.0 * value = {type_spec.factor[0]} / {type_spec.factor[1]} {type_spec.unit}"])
                tbl.append(["Conversion factor", f"1.0 * value = :math:`\\frac{{{type_spec.factor[0]}}}{{{type_spec.factor[1]}}}` {type_spec.unit}"])
            return table(["Property", "Value"], tbl)
        case BoolMeta():
            tbl.extend([
                ["Base type", "bool"],
                ["Bit width", 1],
                ["Default value", meta.default_value],
            ])
            return table(["Property", "Value"], tbl)
        case PadMeta():
            tbl.extend([
                ["Base type", "pad"],
                ["Bit width", type_spec.bits],
            ])
        case BufMeta():
            tbl.extend([
                ["Base type", "buf"],
                ["Bit width", type_spec.bits],
                ["Default value", render_buf_int(meta.default_value)]
            ])
            return table(["Property", "Value"], tbl)
        case BitsetMeta():
            prop_table = table(["Property", "Value"], [
                ["Base type", "bitset"],
                ["Bit width", meta.width],
            ])
            default_table = table(
                ["Flag index", "Flag name", "Default value", "Description"], 
                [[x.bit_idx, x.name, int(x.default_value), x.comment.strip().replace("\n", " ")] for x in meta.flags]
            )

            return f"{prop_table}\n**Flags:**\n{default_table}\n"

        case StructMeta():
            prop_table = table(["Property", "Value"], [
                ["Base type", "struct"],
                ["Bit width", dtype.bit_length()],
            ])

            sig_table = table(['Name', 'Type', 'Default value', 'Description'], [
                [
                    f"``{sig_toml.name}``", 
                    format_dtype(sig_toml.dtype),
                    render_default_value(dev, sig_toml.dtype, sig_toml.default_value, sig.dtype),
                    sig.comment.strip().replace("\n", " ")
                ] for (sig_toml, sig) in zip(type_spec.signals, meta.signals)
            ])

            return f"{prop_table}\n**Signals:**\n{sig_table}\n"
        case aaa:
            raise ValueError(f"Invalid dtype for context encountered: {aaa}")

def render_enum(enum: toml_defs.EnumSpec):
    prop_table = table(["Property", "Value"], [
        ["Bit width", enum.bits],
        ["Default enum", enum.default_value],
    ])


    variant_table = table(
        ["Enum index", "Variant name", "Description"], 
        [[hex(ent.id), name, ent.comment.strip().replace("\n", " ")] for name, ent in enum.values.items()]
    )

    return f"{prop_table}\n**Enum variants:**\n{variant_table}\n"

def gen_spec(dev: toml_defs.DeviceSpec, file: Path):
    rst_root = file.parent/"rst"
    info = read_external_rst(rst_root, dev)

    rendered = env.get_template("spec_template.rst.j2").render(
        hex=hex,
        table=table,
        header=header,
        dev=dev,
        dev_name=dev.name.lower(),
        info=info,
        render_meta_table=render_meta_table,
        render_msg_summary_table=render_msg_summary_table,
        render_msg_table=render_msg_table,
        render_msg_signal_table=render_msg_signal_table,
        render_stg_summary_table=render_stg_summary_table,
        render_setting_table=render_setting_table,
        render_setting_cmd_table=render_setting_cmd_table,
        render_stgcmd_summary_table= render_stgcmd_summary_table,
        render_type = render_type,
        render_enum = render_enum,

        type_to_render = dev.types.items(),
        enum_to_render = [(name, ent) for (name, ent) in dev.enums.items() if name not in ["SETTING"]],
        setting_cmd_to_render=renderable_setting_commands(dev),
        setting_to_render=renderable_settings(dev),
        msg_to_render=renderable_messages(dev), # and name in info['msg']
    )

    keys = []
    dev_header = f".. _{dev.name.lower()}_"
    for line in rendered.splitlines():
        line = line.strip()
        if line.startswith(dev_header) and line.endswith(":"):
            keys.append(line[len(dev_header):-1])
    for key in keys:
        rendered = rendered.replace(f"<{key}>`", f"<{dev.name.lower()}_{key}>`")
    return rendered



if __name__ == "__main__":
    import sys
    largs = len(sys.argv)

    dev: toml_defs.DeviceSpec
    spec: str

    match sys.argv:
        case [exe]:
            print(exe, "file [public|not_public] [devid]")
            exit(1)
        case [exe, file, *rest]:
            file_path = Path(file)
            spec = gen_spec(dev := parse_spec(file_path), file_path)
            #print(spec)
    
    Path("target/rst").mkdir(parents=True, exist_ok=True)
    with open(f"target/rst/{dev.name}.rst", "w") as f:
        f.write(spec)

