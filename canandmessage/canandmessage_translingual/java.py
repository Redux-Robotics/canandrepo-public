import typing
import textwrap
import html
from .canandmessage_parser import *
from .canandmessage_parser import utils
# TODO:
# publicity checking
# enums as proper Java enumers?
# bitset builders
# range based validation for uints, sints, floats, and enums *(kinda done)
# any sort of utype conversions or factors


COPYRIGHT_NOTICE = """// Copyright (c) Redux Robotics and other contributors.
// This is open source and can be modified and shared under the 3-clause BSD license. 

"""
IDENT = "    "
NL = "\n"

def doc_comment(s: str) -> str:
    return f"/**\n{NL.join(' * ' + l for l in html.escape(s, quote=False).splitlines())}\n */"

def sign_extend(expr: str, width: int) -> str:
    if width > 32:
        shift = 64 - width
    else:
        shift = 32 - width
    
    if shift == 0:
        return expr
    return f"(({expr}) << {shift}) >> {shift}"
    
def extract_lbits(expr: str, width: int, offset: int, signed=False) -> str:
    if offset != 0:
        expr = f"({expr} >> {offset})"
    if width == 64:
        return expr
    if width > 31 and not (width == 32 and signed):
        return f"{expr} & 0x{(1 << width) - 1:x}L"
    if width == 32:
        return f"((int) {expr})"
    return f"((int) {expr}) & 0x{(1 << width) - 1:x}"

def get_type_for_dtype(dtype: DType):
    meta = dtype.meta
    if isinstance(meta, SIntMeta) or isinstance(meta, BitsetMeta):
        return "int" if meta.width <= 32 else "long"
    if isinstance(meta, BoolMeta):
        return "boolean"
    if isinstance(meta, FloatMeta):
        return "float" if meta.width <= 32 else "double"
    
    return "int" if dtype.bit_length() < 32 else "long"

def rshift_to_long(expr: str, offset: int) -> str:
    if offset == 0:
        return f"{expr}"
    return f"({expr} << {offset})"

def jtype_to_long(name: str, jtype: str, offset: int, width: int) -> str:
    mask = hex(utils.default_uint_max(width))
    match jtype:
        case 'int':
            # so we need the bitmask to ensure no sign extension happens. because java.
            return rshift_to_long(f"((long) {name})" if width < 32 
                                  else f"((long) {name} & 0xffffffffL)", offset)
        case 'long':
            return rshift_to_long(f"({name} & {mask}L)" if width != 64 else name, offset)
        case 'boolean':
            return rshift_to_long(f"({name} ? 1L : 0L)", offset)
        case 'float':
            if width == 24:
                return rshift_to_long(f"((long) (Float.floatToIntBits({name}) >> 8) & 0xffffffL)", offset)
            if width == 32:
                return rshift_to_long(f"((long) Float.floatToIntBits({name}) & 0xffffffffL)", offset)
            else:
                utils.panic(ValueError(f"float width {width} unsupported"))
        case 'double':
            return rshift_to_long(f"Double.doubleToLongBits({name})", offset)



def gen_cls(name: str, members: typing.List[str], doc: str, visibility="public", modifier="", ctype="class", uninstantiable=False) -> str:
    DELIM = NL*2
    if uninstantiable:
        members = [f"private {name}() {{}}"] + list(members)

    return f"""{doc_comment(doc)}
{' '.join(c for c in [visibility, modifier, ctype] if c)} {name} {{
{DELIM.join(textwrap.indent(m, IDENT) for m in members)}
}}"""

def gen_sig_extract(sig: Signal, prefix="", offset=0, apply_prefix=True) -> typing.Tuple[typing.List[str], int]:
    # extractMessageName_SigName_SubsigName(long field) -> [value]
    extract_value = ""
    meta = sig.dtype.meta
    is_pad_or_none = meta is None
    name = utils.screaming_snake_to_camel(sig.name)
    match meta:
        case UIntMeta() | EnumMeta() | BufMeta():
            extract_value = f"return {extract_lbits('field', meta.width, offset)};"
            offset += meta.width
        case BitsetMeta():
            extract_value = f"return {extract_lbits('field', meta.width, offset, True)};"
            offset += meta.width
        case SIntMeta():
            extract_value = f"return {sign_extend(extract_lbits('field', meta.width, offset, True), meta.width)};"
            offset += meta.width
        case FloatMeta():
            match meta.width:
                case 24:
                    extract_value = f"return Float.intBitsToFloat(({extract_lbits('field', 24, offset)}) << 8);"
                case 32:
                    extract_value = f"return Float.intBitsToFloat({extract_lbits('field', 32, offset, True)});"
                case 64:
                    extract_value = f"return Double.longBitsToDouble(field >> {offset});"
                case _:
                    utils.panic(ValueError(f"float width {meta.width} unsuppoerted in sig {sig.name}"))
            offset += meta.width
        case BoolMeta():
            extract_value = f"return ((field >> {offset}) & 1) > 0;"
            offset += 1
        case PadMeta():
            is_pad_or_none = True
            offset += meta.width
        case StructMeta():
            prefix = f"{prefix if apply_prefix else ''}{name}_"
            extract_value = []
            for subsig in meta.signals:
                v, offset = gen_sig_extract(subsig, prefix, offset)
                extract_value.extend(v)
        case _:
            is_pad_or_none = True
    if is_pad_or_none:
        return [], offset
    if isinstance(extract_value, str):
        doc = doc_comment(f"Extracts {sig.comment} from {prefix.strip('_')}.\n\n"
                        f"@param field data bitfield\n"
                        f"@return {sig.name} as a {sig.dtype.canonical_name()}")
        return [f"""{doc}
public static {get_type_for_dtype(sig.dtype)} extract{prefix if apply_prefix else ''}{name}(long field) {{
{textwrap.indent(extract_value, IDENT)}
}}"""], offset
    else:
        return extract_value, offset

def gen_check(expr: str, err_msg: str):
    return f"if ({expr}) {{ throw new IllegalArgumentException({err_msg}); }}"

def gen_sig_checks(sig: Signal) -> typing.List[str]:
    meta = sig.dtype.meta
    sig_name = utils.snake_to_stilted_camel(sig.name)
    jtype = get_type_for_dtype(sig.dtype)
    l_is_real = "L" if jtype == "long" else ""
    match meta:
        case UIntMeta():
            if meta.width == 64:
                return [] # attempting to bounds-check a 64-bit uint in this godforsaken signed-only lang will end Poorly.
            # min and max are well-defined for the inputs
            return [gen_check(f"{sig_name} < {meta.min}{l_is_real} || {sig_name} > {meta.max}{l_is_real}", 
                              f'"{sig_name} must be between [{meta.min}..={meta.max}] inclusive, instead got " + {sig_name}')]
        case SIntMeta():
            if ((meta.width == 32 or meta.width == 64) and 
                meta.min == utils.default_sint_min(meta.width) and 
                meta.max == utils.default_sint_max(meta.width)):
                return [] # special case the precise instance where field width equals listed min/max
            return [gen_check(f"{sig_name} < {meta.min}{l_is_real} || {sig_name} > {meta.max}{l_is_real}", 
                              f'"{sig_name} must be between [{meta.min}..={meta.max}] inclusive, instead got " + {sig_name}')]
        case BufMeta() | BitsetMeta():
            if meta.width == 64 and jtype == "long" or meta.width == 32 and jtype == "int":
                # bounds check don't work at signed type boundary conditions
                return []
            # check that the value is within 0..uint_max(sig.dtype.bit_length())
            umax = utils.default_uint_max(sig.dtype.bit_length())
            return [gen_check(f"{sig_name} < 0{l_is_real} || {sig_name} > {umax}{l_is_real}", 
                              f'"{sig_name} must be between [0..={umax}] inclusive, instead got " + {sig_name}')]
        case FloatMeta():
            # check min and max if not None, but also check nan/inf
            checks = []
            if meta.min is not None and meta.max is not None:
                checks.append(gen_check(f"{sig_name} < {meta.min} || {sig_name} > {meta.max}", 
                              f'"{sig_name} must be between [{meta.min}..={meta.max}] inclusive, instead got " + {sig_name}'))
            else:
                if meta.min is not None:
                    checks.append(gen_check(f"{sig_name} < {meta.min}",
                                f'"{sig_name} value " + {sig_name} + " violates bound {sig_name} >= {meta.min}"'))
                if meta.max is not None:
                    checks.append(gen_check(f"{sig_name} > {meta.max}",
                                f'"{sig_name} value " + {sig_name} + " violates bound {sig_name} <= {meta.max}"'))
            if not meta.allow_nan_inf:
                checks.append(gen_check(f"!{jtype.capitalize()}.isFinite({sig_name})", f'"{sig_name} cannot be infinite or NaN!"'))
            return checks
        case PadMeta() | BoolMeta():
            # no checks.
            return []
        case EnumMeta():
            # yknow? i could do this. or i could tell you to just use the enum constants and tell you skill issue otherwise.
            # in fact that sounds smarter. skill issue.
            ids = sorted(meta.values.keys())
            cont_ranges: typing.List[int | typing.Tuple[int, int]] = []
            return []
        case StructMeta():
            # recursive structure.
            checks = []
            for subsig in meta.signals:
                checks.extend(gen_sig_checks(Signal(
                    name = sig.name + "_" + subsig.name,
                    comment = subsig.comment,
                    dtype = subsig.dtype,
                    optional = subsig.optional
                )))
            return checks
        case _:
            return []
        


def _render_sig(sig: Signal, offset: int) -> typing.Tuple[typing.List[str], typing.List[str], typing.List[str], int]:
    if isinstance(sig.dtype.meta, PadMeta):
        return [], [], [], offset + sig.dtype.bit_length()
    if isinstance(sig.dtype.meta, StructMeta):
        param, arg, pack_expr = [], [], []
        for subsig in sig.dtype.meta.signals:
            p, a, k, offset = _render_sig(
                Signal(
                    name = sig.name + "_" + subsig.name,
                    comment = subsig.comment,
                    dtype = subsig.dtype,
                    optional=subsig.optional,
                ), offset)
            param.extend(p)
            arg.extend(a)
            pack_expr.extend(k)
        return param, arg, pack_expr, offset

    jtype = get_type_for_dtype(sig.dtype)
    sig_name = utils.snake_to_stilted_camel(sig.name)
    try:
        param = f"@param {sig_name} {sig.comment} ({sig.dtype.canonical_name()})"
    except Exception:
        raise ValueError(str(sig))
    arg = f"{jtype} {sig_name}"
    pack_expr = jtype_to_long(sig_name, jtype, offset, sig.dtype.bit_length())
    return ([param], [arg], [pack_expr], offset + sig.dtype.bit_length())

def gen_sigs_pack(name: str, signals: typing.List[Signal], compound_type: str, check_bounds=False) -> str:
    params = []
    args = []
    pack_exprs = []
    offset = 0
    for sig in signals:
        param, arg, pack_expr, offset = _render_sig(sig, offset)
        params.extend(param)
        args.extend(arg)
        pack_exprs.extend(pack_expr)
    if not pack_exprs:
        pack_exprs = ['0']

    check_val = ""
    if check_bounds:
        checks = []
        for sig in signals:
            checks.extend(gen_sig_checks(sig))
        if checks:
            check_val = textwrap.indent("\n".join(checks), IDENT) + "\n"

    doc = doc_comment(f"Constructs a {name} {compound_type}.\n\n" + 
                      "\n".join(params) + 
                      "\n@return message data as long")
    
    pack = check_val + textwrap.indent(f'return {(" | " + NL).join(pack_exprs)};', 2*IDENT)[4:]
    return f"""{doc}
public static long construct{utils.screaming_snake_to_camel(name)}({", ".join(args)}) {{
{pack}
}}"""


def gen_msg(dev: Device) -> str:
    members = []

    msg_pad = utils.padder_fn(map(utils.screaming_snake_to_kamel, dev.messages.keys()))
    msg: Message
    for name, msg in utils.rsort_by_ent_id(dev.messages):
        if not msg.is_public:
            continue
        members.append(
            f"{doc_comment(msg.comment)}\npublic static final int {msg_pad(utils.screaming_snake_to_kamel(name))} = 0x{msg.id:x};")
    
    for name, msg in utils.rsort_by_ent_id(dev.messages):
        if not msg.is_public:
            continue
        offset = 0
        for sig in msg.signals:
            v, offset = gen_sig_extract(sig, prefix=utils.screaming_snake_to_camel(name) + "_", offset=offset)
            members.extend(v)
    
    for name, msg in utils.rsort_by_ent_id(dev.messages):
        if not msg.is_public:
            continue
        members.append(gen_sigs_pack(name, msg.signals, "message"))
    
    for name, msg in utils.rsort_by_ent_id(dev.messages):
        if not msg.is_public:
            continue
        if msg.min_length == msg.max_length:
            members.append(f"/** {name} message length */\npublic static final int kDlc_{utils.screaming_snake_to_camel(name)} = {msg.min_length};")
            members.append(f"""/**\n * Check if {name} message length is valid.\n * @param dlc length to check\n * @return true if valid\n */
public static boolean checkDlcFor{utils.screaming_snake_to_camel(name)}(int dlc) {{
    return dlc == {msg.min_length};
}}""")
        else:
            members.append(f"/** {name} message min length */\npublic static final int kDlcMin_{utils.screaming_snake_to_camel(name)} = {msg.min_length};")
            members.append(f"/** {name} message max length */\npublic static final int kDlcMax_{utils.screaming_snake_to_camel(name)} = {msg.max_length};")
            members.append(f"""/**\n * Check if {name} message length is valid.\n * @param dlc length to check\n * @return true if valid\n */
public static boolean checkDlcFor{utils.screaming_snake_to_camel(name)}(int dlc) {{
    return dlc >= {msg.min_length} && dlc <= {msg.max_length};
}}""")

    return gen_cls("Msg", members, doc="Messages.", modifier="static", uninstantiable=True)

def gen_vdep_stg_list(dev: Device) -> str:
    all_stg = []
    for name, stg in dev.settings.items():
        #print(stg)
        if not (stg.vdep_setting and stg.vendordep and stg.readable):
            continue
        
        all_stg.append(f"    {utils.screaming_snake_to_kamel(name)},")
    
    return f"""/** List of settings to fetch for. */
public static int settingsAddresses[] = {{
{'\n'.join(all_stg)}
}};"""

def gen_vdep_default_stg(dev: Device) -> str:
    put_stg = []
    for name, stg in dev.settings.items():
        #print(stg)
        if not (stg.vdep_setting and stg.vendordep and stg.readable):
            continue
        default_value_as_bits = stg.dtype.default_value_as_bits()
        if isinstance(default_value_as_bits, float):
            print("Fucked setting default value: ", stg, stg.dtype.meta)

        put_stg.append(f"    stg.put({utils.screaming_snake_to_kamel(name)}, 0x{stg.dtype.default_value_as_bits():x}L);")

    return f"""/** Creates a HashMap of writable default settings. */
public static Map<Integer, Long> defaultSettings; 
static {{
    Map<Integer, Long> stg = new HashMap<>();
{'\n'.join(put_stg)}
    defaultSettings = stg;
}}
"""

def gen_stg(dev: Device) -> str:
    members = []
    stg_pad = utils.padder_fn(map(utils.screaming_snake_to_kamel, dev.settings.keys()))
    stg: Setting
    for name, stg in utils.rsort_by_ent_id(dev.settings):
        if not stg.vendordep:
            continue

        members.append(
            f"{doc_comment(stg.comment)}\npublic static final int {stg_pad(utils.screaming_snake_to_kamel(name))} = 0x{stg.id:x};")

    for name, stg in utils.rsort_by_ent_id(dev.settings):
        if not stg.vendordep:
            continue
        members.extend(gen_sig_extract(Signal.from_stg(name, stg), prefix=utils.screaming_snake_to_camel(name) + "_", apply_prefix = False)[0])

    for name, stg in utils.rsort_by_ent_id(dev.settings):
        if not stg.vendordep:
            continue
        if isinstance(stg.dtype.meta, StructMeta):
            members.append(gen_sigs_pack(name, stg.dtype.meta.signals, "setting", True))
        else:
            members.append(gen_sigs_pack(name, [Signal.from_stg(name, stg)], "setting", True))
    
    members.append(gen_vdep_stg_list(dev))
    members.append(gen_vdep_default_stg(dev))

    return gen_cls("Stg", members, doc="Settings.", modifier="static", uninstantiable=True)

def gen_enumers(dev: Device) -> str:
    members = []
    for name, meta in dev.enums.items():
        if name == "SETTING":
            continue
        enumer_members = []
        enumer_pad = utils.padder_fn(map(lambda v: utils.screaming_snake_to_kamel(v.name), meta.values.values()))
        for eindex, enumer in meta.values.items():
            enumer_members.append(
            f"{doc_comment(enumer.comment)}\npublic static final int {enumer_pad(utils.screaming_snake_to_kamel(enumer.name))} = 0x{enumer.index:x};")
            pass
        members.append(gen_cls(utils.screaming_snake_to_camel(name), enumer_members, f"enum {dev.name}::{name}.", modifier="static", uninstantiable=True))
    
    return gen_cls("Enums", members, doc=f"{dev.name.capitalize()} enums.", modifier="static", uninstantiable=True)

def gen_bitsets(dev: Device) -> str:
    members = []

    for name, meta in dev.bitsets.items():
        jtype = "int" if meta.width <= 32 else "long"
        for ent in meta.flags:
            members.append(doc_comment(f"{name} - {ent.comment}") + 
                           f"\npublic static final {jtype} {utils.screaming_snake_to_kamel(name)}_"
                           f"{utils.screaming_snake_to_camel(ent.name)} = {hex(1 << ent.bit_idx)};")

    for name, meta in dev.bitsets.items():
        jtype = "int" if meta.width <= 32 else "long"
        doc_params = [f"@param {utils.snake_to_stilted_camel(ent.name)} {ent.comment.strip()}" for ent in meta.flags]
        arg_params = [f"boolean {utils.snake_to_stilted_camel(ent.name)}" for ent in meta.flags]
        lfix = 'L' if jtype == 'long' else ''
        pack_exprs = [f"({utils.snake_to_stilted_camel(ent.name)} ? {hex(1 << ent.bit_idx)}{lfix} : 0)" for ent in meta.flags] or ["0"]

        doc = doc_comment(f"Constructs a {name} bitset.\n\n" + 
                        "\n".join(doc_params) + 
                        f"\n@return bitset data as {jtype}")
    
        pack = textwrap.indent(f'return {(" | " + NL).join(pack_exprs)};', 2*IDENT)[4:]
        members.append(f"{doc}\n"
        f"public static {jtype} construct{utils.screaming_snake_to_camel(name)}({', '.join(arg_params)}) {{\n"
        f"{pack}\n}}")
    
    for name, meta in dev.bitsets.items():
        for ent in meta.flags:
            doc = doc_comment(f"Extracts {ent.name} from {name}.\n\n"
                              f"@param field data bitfield\n"
                              f"@return true if set, false if not")
            members.append(f"{doc}\n"
            f"public static boolean extract{utils.screaming_snake_to_camel(name)}_{utils.screaming_snake_to_camel(ent.name)}({jtype} field) {{\n"
            f"{IDENT}return (field & {utils.screaming_snake_to_kamel(name)}_{utils.screaming_snake_to_camel(ent.name)}) > 0;\n"
            f"}}")
    return gen_cls("Bitsets", members, doc=f"{dev.name.capitalize()} bitsets.", modifier="static", uninstantiable=True)
            
def gen_types(dev: Device) -> str:

    for name, spec in dev.spec.types.items():
        pass

    return ""


def gen_details(dev: Device) -> str:
    return (
        COPYRIGHT_NOTICE + 
        f"package {dev.java_package};\n\n" + 
        "import java.util.HashMap;\nimport java.util.Map;\n\n" +
        gen_cls(dev.name + "Details", [
            gen_msg(dev),
            gen_stg(dev),
            gen_enumers(dev),
            gen_bitsets(dev)
        ], doc=f"""{dev.name} device constants. 

This file is autogenerated by canandmessage, <b>do not hand-edit!</b>
""", uninstantiable=True))

if __name__ == "__main__":
    import sys
    import pathlib
    if len(sys.argv) < 3:
        print("usage", sys.argv[0], "[tomlfile] [reduxlib root]")

    in_path = pathlib.Path(sys.argv[1])
    reduxlib_path = pathlib.Path(sys.argv[2])

    dev: Device = parse_spec_to_device(in_path)
    out_path = reduxlib_path/f"src/main/java/{dev.java_package.replace('.', '/')}/{dev.name}Details.java"

    with open(out_path, "w") as f:
        f.write(gen_details(dev))