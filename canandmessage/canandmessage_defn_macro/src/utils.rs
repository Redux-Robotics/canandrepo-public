use canandmessage_parser::{DType, Device};
use proc_macro2::Literal;
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn screaming_snake_to_camel(s: &str) -> String {
    s.split('_')
        .by_ref()
        .map(|v| capitalize(v.to_lowercase().as_str()))
        .collect::<String>()
}

pub fn screaming_snake_to_ident(s: &String) -> Ident {
    format_ident!("{}", crate::utils::screaming_snake_to_camel(s.as_str()))
}

pub fn u_with_size(size: usize) -> proc_macro2::TokenStream {
    format_ident!("u{}", min_width(size)).into_token_stream()
}

pub fn i_with_size(size: usize) -> proc_macro2::TokenStream {
    format_ident!("i{}", min_width(size)).into_token_stream()
}

pub fn f_with_size(size: usize) -> proc_macro2::TokenStream {
    format_ident!("f{}", min_width(size)).into_token_stream()
}

pub fn u8_buf(size: usize) -> proc_macro2::TokenStream {
    let bufsz = Literal::usize_unsuffixed((size + 7) >> 3);
    quote! {[u8; #bufsz]}
}

pub fn gen_can_id(device: &Device, msg_id: u8) -> u32 {
    let mut out = 0;
    if device.dev_type != 31 {
        out = (device.dev_type as u32) << 24;
    };

    out |= (0xE) << 16;
    //out |= (device.dev_class as u32) << 10;
    out | (msg_id as u32) << 6
}

pub fn min_width(bits: usize) -> usize {
    match bits {
        0..=8 => 8,
        9..=16 => 16,
        17..=32 => 32,
        33..=64 => 64,
        65.. => 64,
        _ => todo!(),
    }
}

pub fn flatten_token_vec(tok_vec: Vec<proc_macro2::TokenStream>) -> proc_macro2::TokenStream {
    quote! { #(#tok_vec)* }
}

fn fully_qualified_type_name(dev_name: &String, name: &String) -> proc_macro2::TokenStream {
    let ident = crate::utils::screaming_snake_to_ident(name);
    let lname = format_ident!("{}", dev_name.to_lowercase());
    quote! { crate::#lname::types::#ident }
}

/// The canonical function for translating a DType into a generated type name.
/// None and Pad canonically return None.
pub fn gen_type_for_dtype(dev: &Device, dtype: &DType) -> Option<proc_macro2::TokenStream> {
    match dtype {
        DType::None => None,
        DType::UInt { meta } => Some(u_with_size(meta.width)),
        DType::SInt { meta } => Some(i_with_size(meta.width)),
        DType::Buf { meta } => Some(u8_buf(meta.width)),
        DType::Float { meta } => Some(f_with_size(meta.width)),
        DType::Bitset { meta } => Some(fully_qualified_type_name(&dev.name, &meta.name)),
        DType::Pad { width } => None,
        DType::Bool { default_value } => Some(quote!(bool)),
        DType::Enum { meta } => Some(fully_qualified_type_name(&dev.name, &meta.name)),
        DType::Struct { meta } => Some(fully_qualified_type_name(&dev.name, &meta.name)),
    }
}

pub fn byte_aligned(sz: usize) -> bool {
    sz % 8 == 0
}

pub fn lname(device: &Device) -> proc_macro2::TokenStream {
    format_ident!("{}", device.name.to_lowercase()).to_token_stream()
}

/// unlike Literal::f64_unsuffixed, this allows for nan/inf
pub fn float_literal(f: f64, width: usize) -> proc_macro2::TokenStream {
    let ftype = f_with_size(width);
    if f.is_nan() {
        quote!(#ftype::NAN)
    } else if f.is_infinite() {
        if f.is_sign_positive() {
            quote!(#ftype::INFINITY)
        } else {
            quote!(#ftype::NEG_INFINITY)
        }
    } else {
        match width {
            16..=32 => Literal::f32_suffixed(f as f32),
            64 => Literal::f64_suffixed(f),
            _ => panic!("only 32 and 64-bit floats supported"),
        }
        .to_token_stream()
    }
}

pub fn uint_literal(u: u64, width: usize) -> proc_macro2::TokenStream {
    if u > canandmessage_parser::utils::default_uint_max(width) {
        panic!("uint {} is too large for width {}", u, width);
    }

    match width {
        0..=8 => Literal::u8_suffixed(u as u8),
        9..=16 => Literal::u16_suffixed(u as u16),
        17..=32 => Literal::u32_suffixed(u as u32),
        33..=64 => Literal::u64_suffixed(u),
        65.. => Literal::u64_suffixed(u),
        _ => unreachable!("uint decode failed somehow"),
    }
    .to_token_stream()
}

pub fn sint_literal(i: i64, width: usize) -> proc_macro2::TokenStream {
    if i > canandmessage_parser::utils::default_sint_max(width)
        || i < canandmessage_parser::utils::default_sint_min(width)
    {
        panic!("sint {} is too large for width {}", i, width);
    }

    match width {
        0..=8 => Literal::i8_suffixed(i as i8),
        9..=16 => Literal::i16_suffixed(i as i16),
        17..=32 => Literal::i32_suffixed(i as i32),
        33..=64 => Literal::i64_suffixed(i),
        65.. => Literal::i64_suffixed(i),
        _ => unreachable!("sint decode failed somehow"),
    }
    .to_token_stream()
}
pub fn buf_literal(v: &[u8], width: usize) -> proc_macro2::TokenStream {
    let bwidth = (width + 7) / 8;
    let u8_ents: Vec<Literal> = v[0..bwidth]
        .iter()
        .map(|u8v| Literal::u8_suffixed(*u8v))
        .collect();
    quote!([#(#u8_ents),*])
}
