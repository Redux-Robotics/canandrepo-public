#![allow(unused, dead_code)]
use canandmessage_parser::utils as putils;
use canandmessage_parser::DType;
use canandmessage_parser::Device;
use canandmessage_parser::Signal;

const COPYRIGHT_NOTICE: &str = "// Copyright (c) Redux Robotics and other contributors.
// This is open source and can be modified and shared under the 3-clause BSD license. 

";

const INDENT: &str = "    ";
const NL: &str = "\n";

/// Generates a doc comment.
fn doc_comment(s: &String) -> String {
    let body = s
        .split_terminator('\n')
        .map(|line| format!(" * {line}"))
        .collect::<Vec<String>>()
        .join("\n");
    format!("/**\n{body}\n */")
}

/// Generates a sign extension expression for a signed integer field.
fn sign_extend(expr: &String, width: usize) -> String {
    let shift = if width > 32 { 64 - width } else { 32 - width };

    if shift == 0 {
        expr.clone()
    } else {
        format!("(({expr}) << {shift}) >> {shift}")
    }
}

/// Generates the bit manipulation expression to extract a signal from a long field.
fn extract_lbits(expr: &String, width: usize, offset: usize, signed: bool) -> String {
    let expr = if offset != 0 {
        format!("({expr} >> {offset})")
    } else {
        expr.clone()
    };

    if width == 64 {
        return expr;
    }
    if width > 31 && !(width == 32 && signed) {
        return format!("{expr} & 0x{:x}L", (1 << width) - 1);
    }
    if width == 32 {
        return format!("((int) {expr})");
    }
    format!("((int) {expr}) & 0x{:x}", (1 << width) - 1)
}

/// Gets the Java type string for the dtype based on width
fn get_type_for_dtype(dtype: &DType) -> String {
    match dtype {
        DType::None => unreachable!(),
        DType::Bitset { .. } | DType::SInt { .. } => {
            if dtype.bit_length() <= 32 {
                "int"
            } else {
                "long"
            }
        }
        DType::Bool { .. } => "boolean",
        DType::Float { meta } => {
            if meta.width <= 32 {
                "float"
            } else {
                "double"
            }
        }
        _ => {
            if dtype.bit_length() < 32 {
                "int"
            } else {
                "long"
            }
        }
    }
    .to_string()
}

fn rshift_to_long(expr: &String, offset: usize) -> String {
    if offset == 0 {
        expr.clone()
    } else {
        format!("({expr} << {offset})")
    }
}

/// Generates a conversion from a jtype to the formatted long.
fn jtype_to_long(name: &String, jtype: &String, offset: usize, width: usize) -> String {
    let mask = format!("{:x}", putils::default_uint_max(width));

    let expr = match jtype.as_str() {
        "int" => {
            if width < 32 {
                format!("((long) {name})")
            } else {
                format!("((long) {name} & 0xffffffffL)")
            }
        }
        "long" => {
            if width != 64 {
                format!("({name} & {mask}L)")
            } else {
                name.clone()
            }
        }
        "boolean" => format!("({name} ? 1L : 0L)"),
        "float" => match width {
            24 => format!("((long) (Float.floatToIntBits({name}) >> 8) & 0xffffffL)"),
            32 => format!("((long) Float.floatToIntBits({name}) & 0xffffffffL)"),
            _ => panic!("float width {width} unsupported"),
        },
        "double" => format!("Double.doubleToLongBits({name})"),
        _ => panic!("unsupported jtype {jtype}"),
    };
    rshift_to_long(&expr, offset)
}

fn screaming_snake_to_kamel(s: &String) -> String {
    let joined = screaming_snake_to_camel(s);
    format!("k{joined}")
}

fn screaming_snake_to_camel(s: &String) -> String {
    s.split('_')
        .map(putils::capitalize)
        .collect::<Vec<String>>()
        .concat()
}

fn snake_to_stilted_camel(s: &String) -> String {
    s.split('_')
        .enumerate()
        .map(|(i, c)| {
            if i == 0 {
                c.to_lowercase().to_string()
            } else {
                putils::capitalize(c)
            }
        })
        .collect::<Vec<String>>()
        .concat()
}

//fn rsort_by_ent_id()

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
enum Visibility {
    Private,
    Protected,
    PackageLocal,
    Public,
}
impl Visibility {
    pub fn to_str(&self) -> &str {
        match self {
            Visibility::Private => "private",
            Visibility::Protected => "protected",
            Visibility::PackageLocal => "",
            Visibility::Public => "public",
        }
    }

    pub fn apply_modifier(&self, s: &String) -> String {
        match self {
            Visibility::PackageLocal => s.clone(),
            _ => format!("{} {}", self.to_str(), s),
        }
    }
}

fn gen_cls(
    name: &String,
    members: &Vec<String>,
    doc: &String,
    visibility: Visibility,
    decltype: &str,
    uninstantiable: bool,
) -> String {
    let members = if uninstantiable {
        let mut m2 = members.clone();
        m2.insert(0, format!("private {name}() {{}}"));
        m2
    } else {
        members.clone()
    };

    format!(
        "{}
{} {name} {{
{}
}}",
        doc_comment(doc),
        visibility.apply_modifier(&decltype.to_string()),
        members
            .iter()
            .map(|m| INDENT.to_string() + m)
            .collect::<Vec<String>>()
            .join("\n\n")
    )
}

fn gen_sig_extract(
    sig: &Signal,
    prefix: &String,
    offset: usize,
    apply_prefix: bool,
) -> (Vec<String>, usize) {
    let name = screaming_snake_to_camel(&sig.name);
    let new_off = offset + sig.dtype.bit_length();
    let field = "field".to_string();
    let extract = match &sig.dtype {
        DType::None => return (Vec::new(), offset),
        DType::Pad { .. } => return (Vec::new(), new_off),
        DType::UInt { .. } | DType::Enum { .. } | DType::Buf { .. } => {
            let width = sig.dtype.bit_length();
            format!("return {}", extract_lbits(&field, width, offset, false))
        }
        DType::Bitset { meta } => {
            format!("return {}", extract_lbits(&field, meta.width, offset, true))
        }
        DType::SInt { meta } => {
            format!(
                "return {}",
                sign_extend(&extract_lbits(&field, meta.width, offset, true), meta.width)
            )
        }
        DType::Float { meta } => match meta.width {
            24 => format!(
                "return Float.intBitsToFloat(({}) << 8);",
                extract_lbits(&field, 24, offset, false)
            ),
            32 => format!(
                "return Float.intBitsToFloat({});",
                extract_lbits(&field, 32, offset, true)
            ),
            64 => format!("return Double.longBitsToDouble(field >> {offset});"),
            _ => panic!(
                "float width {} unsuppoerted in sig {}",
                meta.width, sig.name
            ),
        },
        DType::Bool { .. } => format!("return ((field >> {offset}) & 1) > 0;"),
        DType::Struct { meta } => {
            let prefix = if apply_prefix {
                format!("{prefix}{name}_")
            } else {
                format!("{name}_")
            };
            let mut new_offset = offset;
            let extract_value = meta
                .signals
                .iter()
                .map(|subsig| {
                    let (v, new_off) = gen_sig_extract(subsig, &prefix, new_offset, true);
                    new_offset = new_off;
                    v
                })
                .flatten()
                .collect::<Vec<String>>();
            return (extract_value, new_offset);
        }
    };
    (
        vec![format!(
            "Extracts {sig_comment} from {sig_prefix}.

        @param field data bitfield
        @return {sig_name} as a {canon_name}
        public static {return_type} extract{applied_prefix}{name}(long field) {{
        {body}
        }}",
            sig_comment = sig.comment,
            sig_prefix = prefix.trim_matches('_'),
            sig_name = sig.name,
            canon_name = sig.dtype.canonical_name(),
            return_type = get_type_for_dtype(&sig.dtype),
            applied_prefix = if apply_prefix { prefix.as_str() } else { "" },
            body = putils::indent(&extract, INDENT)
        )],
        offset,
    )
}

fn gen_check(expr: &String, err_msg: &String) -> String {
    format!("if ({expr}) {{ throw new IllegalArgumentException({err_msg}); }}")
}

fn gen_sig_checks(sig: &Signal) -> Vec<String> {
    let sig_name = snake_to_stilted_camel(&sig.name);
    let jtype = get_type_for_dtype(&sig.dtype);
    let l_is_real = if jtype.as_str() == "long" { "L" } else { "" };
    match &sig.dtype {
        DType::UInt { meta } => {
            // attempting to bounds-check a 64-bit uint in this godforsaken signed-only lang will end Poorly.
            if meta.width == 64 {
                return Vec::new();
            }
            let (min, max) = (
                meta.min.unwrap_or(0),
                meta.max.unwrap_or(putils::default_uint_max(meta.width)),
            );
            vec![gen_check(
                &format!("{sig_name} < {min}{l_is_real} || {sig_name} > {max}{l_is_real}"), 
                &format!("\"{sig_name} must be between [{min}..={max}] inclusive, instead got \", + {sig_name}")
            )]
        }
        DType::SInt { meta } => {
            let min = meta.min.unwrap_or(putils::default_sint_min(meta.width));
            let max = meta.max.unwrap_or(putils::default_sint_max(meta.width));
            if (meta.width == 32 || meta.width == 64)
                && min == putils::default_sint_min(meta.width)
                && max == putils::default_sint_max(meta.width)
            {
                // special case the precise instance where field width equals listed min/max
                return Vec::new();
            }

            vec![gen_check(
                &format!("{sig_name} < {min}{l_is_real} || {sig_name} > {max}{l_is_real}"), 
                &format!("\"{sig_name} must be between [{min}..={max}] inclusive, instead got \", + {sig_name}")
            )]
        }
        DType::Buf { .. } | DType::Bitset { .. } => {
            let width = sig.dtype.bit_length();
            if (width == 64 && l_is_real == "L") || (width == 32 && jtype.as_str() == "int") {
                // bounds check don't work at signed type boundary conditions
                return Vec::new();
            }
            let umax = putils::default_uint_max(width);
            vec![gen_check(
                &format!("{sig_name} < 0{l_is_real} || {sig_name} > {umax}{l_is_real}"), 
                &format!("\"{sig_name} must be between [0..={umax}] inclusive, instead got \" + {sig_name}\"")
            )]
        }
        DType::Float { meta } => {
            let mut checks: Vec<String> = Vec::new();
            if let (Some(min), Some(max)) = (meta.min, meta.max) {
                checks.push(gen_check(
                    &format!("{sig_name} < {min} || {sig_name} > {max}"),
                    &format!("\"{sig_name} must be between [{min}..={max}] inclusive, instead got \", + {sig_name}")
                ));
            } else {
                if let Some(min) = meta.min {
                    checks.push(gen_check(
                        &format!("{sig_name} < {min}"),
                        &format!("\"{sig_name} value \" + {sig_name} + \" violates bound {sig_name} >= {min}\"")
                    ));
                }
                if let Some(max) = meta.max {
                    checks.push(gen_check(
                        &format!("{sig_name} < {max}"),
                        &format!("\"{sig_name} value \" + {sig_name} + \" violates bound {sig_name} <= {max}\"")
                    ));
                }
            }
            if !meta.allow_nan_inf {
                checks.push(gen_check(
                    &format!(
                        "{box_class}.isFinite({sig_name})",
                        box_class = putils::capitalize(&jtype)
                    ),
                    &format!("\"{sig_name} cannot be infinite or NaN!\""),
                ));
            }
            checks
        }
        DType::Pad { .. } => Vec::new(),
        DType::Bool { .. } => Vec::new(),
        DType::None => Vec::new(),
        DType::Enum { .. } => Vec::new(),
        DType::Struct { meta } => meta
            .signals
            .iter()
            .map(|subsig| {
                gen_sig_checks(&Signal {
                    name: format!("{}_{}", sig.name, subsig.name),
                    comment: subsig.comment.clone(),
                    dtype: subsig.dtype.clone(),
                    optional: subsig.optional,
                })
            })
            .flatten()
            .collect(),
    }
}

fn render_sig(sig: &Signal, offset: usize) -> (Vec<String>, Vec<String>, Vec<String>, usize) {
    match &sig.dtype {
        DType::Pad { width } => {
            return (Vec::new(), Vec::new(), Vec::new(), offset + *width);
        }
        DType::Struct { meta } => {
            let (mut param, mut arg, mut pack_expr) = (Vec::new(), Vec::new(), Vec::new());
            let mut new_offset = offset;
            for subsig in &meta.signals {
                let (mut p, mut a, mut k, o) = render_sig(
                    &Signal {
                        name: format!("{}_{}", sig.name, subsig.name),
                        comment: subsig.comment.clone(),
                        dtype: subsig.dtype.clone(),
                        optional: subsig.optional,
                    },
                    new_offset,
                );

                param.append(&mut p);
                arg.append(&mut a);
                pack_expr.append(&mut k);
                new_offset = o;

                return (param, arg, pack_expr, new_offset);
            }
        }
        _ => (),
    };

    let jtype = get_type_for_dtype(&sig.dtype);
    let sig_name = snake_to_stilted_camel(&sig.name);
    let param = format!(
        "@param {sig_name} {sig_comment} ({sig_dname})",
        sig_comment = sig.comment,
        sig_dname = sig.dtype.canonical_name()
    );
    let arg = format!("{jtype} {sig_name}");
    let width = sig.dtype.bit_length();
    let pack_expr = jtype_to_long(&sig_name, &jtype, offset, width);
    (vec![param], vec![arg], vec![pack_expr], offset + width)
}

fn gen_sigs_pack(
    name: &String,
    signals: &Vec<Signal>,
    compound_type: &str,
    check_bounds: bool,
) -> String {
    let (mut params, mut args, mut pack_exprs, mut offset) =
        (Vec::new(), Vec::new(), Vec::new(), 0usize);
    for sig in signals {
        let (mut p, mut a, mut k, o) = render_sig(sig, offset);
        params.append(&mut p);
        args.append(&mut a);
        pack_exprs.append(&mut k);
        offset = o;
    }
    if pack_exprs.len() == 0 {
        pack_exprs.push("0".to_string());
    }

    let check_code = if check_bounds {
        let checks = signals
            .iter()
            .map(gen_sig_checks)
            .flatten()
            .collect::<Vec<String>>()
            .join("\n");
        format!("{}\n", putils::indent(&checks, INDENT))
    } else {
        "".to_string()
    };
    let pack = format!(
        "{check_code}{}",
        putils::indent(
            &format!("return {exprs};", exprs = pack_exprs.join(" | \n")),
            "        "
        )
        .split_off(4)
    );

    format!(
        "Constructs a {name} {compound_type}.

        {jparams}
        @return message data as long
        public static long construct{camel_name}({jargs}) {{
        {pack}
        }}",
        jparams = params.join("\n"),
        camel_name = screaming_snake_to_camel(name),
        jargs = args.join(", ")
    )
}

fn gen_msg(dev: &Device) -> String {
    let mut members: Vec<String> = Vec::new();
    let mut index_members: Vec<String> = Vec::new();
    let mut sig_extract_members: Vec<String> = Vec::new();
    let mut sig_pack_members: Vec<String> = Vec::new();
    let mut dlc_members: Vec<String> = Vec::new();
    let pad = dev
        .messages
        .iter()
        .map(|(name, _)| screaming_snake_to_kamel(name))
        .max_by(|k1, k2| k1.len().cmp(&k2.len()))
        .map_or(0_usize, |k| k.len());

    let mut msg_vec = dev
        .messages
        .iter()
        .collect::<Vec<(&String, &canandmessage_parser::Message)>>();
    msg_vec.sort_by(|nm0, nm1| (u8::MAX - nm0.1.id).cmp(&(u8::MAX - nm1.1.id)));
    for (name, msg) in msg_vec {
        if !msg.is_public {
            continue;
        }
        let kamel_name = screaming_snake_to_kamel(name);
        let camel_name = screaming_snake_to_camel(name);

        index_members.push(format!(
            "/** {} */\npublic static final int {kamel_name:<pad$} = 0x{:x}",
            msg.comment, msg.id
        ));

        let mut offset = 0;
        for sig in &msg.signals {
            let (v, offset2) = gen_sig_extract(sig, &format!("{camel_name}_"), offset, true);
            sig_extract_members.extend_from_slice(v.as_slice());
            offset = offset2;
        }

        sig_pack_members.push(gen_sigs_pack(name, &msg.signals, "message", false));

        if msg.min_length == msg.max_length {
            dlc_members.push(format!(
                "/** {name} message length */\npublic static final int kDlc_{camel_name} = {};",
                msg.min_length
            ));
        } else {
            dlc_members.push(format!(
                "/** {name} message length */\npublic static final int kDlcMin_{camel_name} = {};",
                msg.min_length
            ));
            dlc_members.push(format!(
                "/** {name} message length */\npublic static final int kDlcMax_{camel_name} = {};",
                msg.max_length
            ));
        }
    }

    members.append(&mut index_members);
    members.append(&mut sig_extract_members);
    members.append(&mut sig_pack_members);
    members.append(&mut dlc_members);
    gen_cls(
        &"Msg".to_owned(),
        &index_members,
        &"Messages".to_owned(),
        Visibility::Public,
        "static class",
        true,
    )
}

fn gen_stg(dev: &Device) -> String {
    let pad = dev
        .settings
        .iter()
        .map(|(name, _)| screaming_snake_to_kamel(name))
        .max_by(|k1, k2| k1.len().cmp(&k2.len()))
        .map_or(0_usize, |k| k.len());

    let mut msg_vec = dev
        .settings
        .iter()
        .collect::<Vec<(&String, &canandmessage_parser::Setting)>>();
    msg_vec.sort_by(|nm0, nm1| (u8::MAX - nm0.1.id).cmp(&(u8::MAX - nm1.1.id)));

    let mut members: Vec<String> = Vec::new();
    let mut index_members = Vec::new();
    let mut sig_extract_members = Vec::new();
    let mut sig_pack_members = Vec::new();

    for (name, stg) in msg_vec {
        let kamel_name = screaming_snake_to_kamel(name);
        let camel_name = screaming_snake_to_camel(name);
        index_members.push(format!(
            "/** {} */\npublic static final int {kamel_name:<pad$} = 0x{:x}",
            stg.comment, stg.id
        ));
        sig_extract_members.extend(
            gen_sig_extract(
                &Signal::from_stg(name, stg),
                &format!("{camel_name}_"),
                0,
                false,
            )
            .0,
        );

        match &stg.dtype {
            DType::Struct { meta } => {
                sig_pack_members.push(gen_sigs_pack(name, &meta.signals, "setting", true));
            }
            _ => {
                sig_pack_members.push(gen_sigs_pack(
                    name,
                    &vec![Signal::from_stg(name, stg)],
                    "setting",
                    true,
                ));
            }
        }
    }
    members.append(&mut index_members);
    members.append(&mut sig_extract_members);
    members.append(&mut sig_pack_members);

    gen_cls(
        &"Stg".to_owned(),
        &members,
        &"Settings.".to_owned(),
        Visibility::Public,
        "static class",
        true,
    )
}

fn gen_enumers(dev: &Device) -> String {
    let members = dev
        .enums
        .iter()
        .filter_map(|(name, meta)| {
            if name.as_str() == "SETTING" {
                return None;
            }
            let pad = meta
                .values
                .iter()
                .map(|(id, ent)| screaming_snake_to_kamel(&ent.name))
                .max_by(|k1, k2| k1.len().cmp(&k2.len()))
                .map_or(0_usize, |k| k.len());
            let enumer_members = meta
                .values
                .iter()
                .map(|(id, ent)| {
                    let kamel_name = screaming_snake_to_kamel(&ent.name);
                    format!(
                        "/** {c} */\npublic static final int {kamel_name:<pad$} = 0x{id:x};",
                        c = ent.comment
                    )
                })
                .collect::<Vec<String>>();
            Some(gen_cls(
                &screaming_snake_to_camel(name),
                &enumer_members,
                &format!("enum {dev_name}::{name}", dev_name = dev.name),
                Visibility::Public,
                "static class",
                true,
            ))
        })
        .collect();

    gen_cls(
        &"Enums".to_string(),
        &members,
        &format!("{} enums.", putils::capitalize(&dev.name)),
        Visibility::Public,
        "static class",
        true,
    )
}

fn gen_bitsets(dev: &Device) -> String {
    let mut members = Vec::new();
    let mut flag_members = Vec::new();
    let mut pack_members = Vec::new();
    let mut extract_members = Vec::new();

    for (name, meta) in &dev.bitsets {
        let (jtype, lfix) = if meta.width <= 32 {
            ("int", "")
        } else {
            ("long", "L")
        };
        for ent in &meta.flags {
            // flag members
            let kamel_name = screaming_snake_to_kamel(&name);
            let camel_name = screaming_snake_to_camel(&name);
            let camel_ent = screaming_snake_to_camel(&ent.name);
            //let kamel_ent= screaming_snake_to_kamel(&ent.name);
            let ent_name = &ent.name;

            flag_members.push(format!(
                "{doc}\npublic static final {jtype} {kamel_name}_{camel_ent} = {id:x};",
                doc = doc_comment(&format!("{name} - {}", ent.comment)),
                id = (1 << ent.bit_idx)
            ));

            // extract members
            extract_members.push(format!(
                "{doc}\npublic static boolean extract{camel_name}_{camel_ent}({jtype} field) {{
                {INDENT} return (field & {kamel_name}_{camel_name}) > 0;\n}}", 
                doc = doc_comment(&format!("Extracts {ent_name} from {name}.\n\n@param field data bitfield\n@return true if set, false if not"))
            ));
        }
        // pack members
        let doc_params = meta
            .flags
            .iter()
            .map(|ent| {
                format!(
                    "@param {} {}",
                    snake_to_stilted_camel(&ent.name),
                    ent.comment.trim()
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        let arg_params = meta
            .flags
            .iter()
            .map(|ent| format!("boolean {}", snake_to_stilted_camel(&ent.name)))
            .collect::<Vec<String>>()
            .join(", ");

        let pack_exprs = meta
            .flags
            .iter()
            .map(|ent| {
                format!(
                    "({} ? {:x}{} : 0)",
                    snake_to_stilted_camel(&ent.name),
                    (1 << ent.bit_idx),
                    lfix
                )
            })
            .collect::<Vec<String>>()
            .join(" | \n");

        let doc = doc_comment(&format!(
            "Construcst a {name} bitset.\n\n{doc_params}\n@return bitset data as {jtype}"
        ));
        let pack = putils::indent(&format!("return {pack_exprs};"), "        ");
        pack_members.push(format!(
            "{doc}\npublic static {jtype} constrct{camel}({arg_params}) {{\n{pack}\n}}",
            camel = screaming_snake_to_camel(&name)
        ));
    }

    members.append(&mut flag_members);
    members.append(&mut pack_members);
    members.append(&mut extract_members);
    gen_cls(
        &"Bitsets".to_owned(),
        &members,
        &format!("{} bitsets.", putils::capitalize(&dev.name)),
        Visibility::Public,
        "static class",
        true,
    )
}

fn gen_details(dev: &Device) -> String {
    format!("{COPYRIGHT_NOTICE}package {jpkg}\n\n{cls}", 
        jpkg = dev.java_package,
        cls = gen_cls(&format!("{}Details", dev.name), &vec![
            gen_msg(dev),
            gen_stg(dev),
            gen_enumers(dev),
            gen_bitsets(dev)
        ], 
        &format!("{} device constants.\n\nThis file is autogenerated by canandmessage, <b>do not hand-edit!</b>\n", dev.name), 
        Visibility::Public,
        "static class",
        true)
    )
}
