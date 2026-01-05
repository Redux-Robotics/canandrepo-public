use canandmessage_parser::toml_defs::TypeSpec;
use canandmessage_parser::{DType, Device, Message, Signal, Source, StructMeta};
use proc_macro2::{Ident, Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};

use crate::utils::{self, u_with_size};

pub fn gen_message_enum(device: &Device) -> TokenStream {
    let entries: Vec<TokenStream> = device
        .messages
        .iter()
        .map(|(name, msg)| {
            let msg_name = utils::screaming_snake_to_ident(name);
            let doc_str = Literal::string(msg.comment.as_str());
            let signals: Vec<TokenStream> = msg
                .signals
                .iter()
                .filter_map(|sig| {
                    let sig_dtype = match utils::gen_type_for_dtype(device, &sig.dtype) {
                        Some(v) => {
                            if sig.optional {
                                quote! { Option<#v> }
                            } else {
                                v
                            }
                        }
                        None => return None,
                    };
                    let sig_doc = Literal::string(sig.comment.as_str());
                    let sig_name = format_ident!("{}", sig.name);
                    Some(quote! {
                        #[doc=#sig_doc]
                        #sig_name: #sig_dtype
                    })
                })
                .collect();

            let msg_id = utils::uint_literal(msg.id as u64, 8);

            // the enum entry
            quote! {
                #[doc=#doc_str]
                #msg_name {
                    #(#signals),*
                } = #msg_id
            }
        })
        .collect();
    quote! {
        #[cfg_attr(feature="device",derive(defmt::Format))]
        #[repr(u8)]
        #[derive(Debug)]
        pub enum Message {
            #(#entries),*
        }
    }
}

fn gen_sig_bit_load(sig: &Signal, dtype: TokenStream, idx: &mut usize) -> TokenStream {
    let width = sig.dtype.bit_length();
    let (start, end) = (*idx, *idx + width);
    let (start_byte, end_byte) = (start / 8, end / 8);
    // increment the idx ctr here.
    *idx += width;

    let backing_int = match sig.dtype {
        DType::SInt { meta: _ } => utils::i_with_size(width),
        _ => utils::u_with_size(width),
    };
    let slice_expr = quote!(data[#start_byte..#end_byte].try_into().unwrap());
    let integral_expr = quote!(bits.get_unchecked(#start..#end).load_le::<#backing_int>());

    let (from_slice, from_bits) = match &sig.dtype {
        DType::UInt { .. } => (quote!(#dtype::from_le_bytes(#slice_expr)), integral_expr),
        DType::SInt { .. } => (quote!(#dtype::from_le_bytes(#slice_expr)), integral_expr),
        DType::Float { meta } => match meta.width {
            32 | 64 => (
                quote!(#dtype::from_le_bytes(#slice_expr)),
                quote!(#dtype::from_bits(#integral_expr)),
            ),
            24 => (
                quote!(f32::from_bits(crate::u24_from_le_bytes(#slice_expr) << 8)),
                quote!(f32::from_bits(#integral_expr << 8)),
            ),
            _ => panic!("unsupported float width"),
        },
        DType::Buf { .. } => (slice_expr, quote!(#integral_expr.to_le_bytes())),
        DType::Enum { .. } => (
            quote!(#dtype::try_from(#backing_int::from_le_bytes(#slice_expr))?),
            quote!(#dtype::try_from(#integral_expr)?),
        ),
        DType::Bitset { .. } => {
            let arb_ubits = format_ident!("u{}", width);
            (
                quote!(#dtype::from_bitfield(#backing_int::from_le_bytes(#slice_expr))),
                quote!(#dtype::from_bitfield(#arb_ubits::from(#integral_expr))),
            )
        }
        _ => unreachable!(),
    };

    if utils::byte_aligned(width) && utils::byte_aligned(start) {
        // if both the width is aligned and the starting point is byte aligned, we just do a byte copy (happy path).
        // this _ideally_ compiles to some memcpy intrinsic
        quote!(unsafe{#from_slice})
    } else {
        // otherwise we convert to some sort of integral type and load it that way
        quote!(unsafe{#from_bits})
    }
}

fn gen_bounds_check(name: TokenStream, sig: &Signal) -> TokenStream {
    let (min, max) = match &sig.dtype {
        DType::UInt { meta } => (
            meta.min.map(Literal::u64_unsuffixed),
            meta.max.map(Literal::u64_unsuffixed),
        ),
        DType::SInt { meta } => (
            meta.min.map(Literal::i64_unsuffixed),
            meta.max.map(Literal::i64_unsuffixed),
        ),
        // this will panic on nan/inf which....shouldn't be valid bounds anyway
        DType::Float { meta } => (
            meta.min.map(Literal::f64_unsuffixed),
            meta.max.map(Literal::f64_unsuffixed),
        ),
        _ => return quote!(),
    };
    let mut check_conditions: Vec<TokenStream> = Vec::new();
    min.map_or((), |v| check_conditions.push(quote!(#name < #v)));
    max.map_or((), |v| check_conditions.push(quote!(#name > #v)));
    match &sig.dtype {
        DType::Float { meta } => {
            if !meta.allow_nan_inf {
                check_conditions.push(quote!(!#name.is_finite()))
            }
        }
        _ => (),
    }

    if check_conditions.len() > 0 {
        quote! {
            if #(#check_conditions)||* {
                return Err(());
            }
        }
    } else {
        quote!()
    }
}

fn gen_assignment(
    name: TokenStream,
    sig: &Signal,
    value: TokenStream,
    idx: usize,
    check_bounds: bool,
) -> (TokenStream, Ident, TokenStream) {
    let expr_name = format_ident!("{}", sig.name.to_owned());
    let idx_bytes = (idx + 7) / 8;

    let declr = if sig.optional {
        let guts = if check_bounds {
            let bounds_check = gen_bounds_check(quote!(check_tmp), sig);
            quote! {
                let check_tmp = #value;
                #bounds_check
                Some(check_tmp)
            }
        } else {
            quote!(Some(#value))
        };
        quote! {
            let #name = (if (dlc >= #idx_bytes) {
                #guts
            } else { None });
        }
    } else {
        let bounds_check = if check_bounds {
            gen_bounds_check(name.clone(), sig)
        } else {
            quote!()
        };
        quote! { let #name = #value; #bounds_check }
    };
    //let struct_fill = quote! { #expr_name: #name, };
    (declr, expr_name, name)
}

pub fn gen_signal_unpacker(
    device: &Device,
    sig: &Signal,
    prefix: String,
    idx: &mut usize,
    check_bounds: bool,
) -> Option<(TokenStream, Ident, TokenStream)> {
    // .0: the declaration/consumption code. .1: the struct filling code.

    let name = format_ident!("{}_{}", prefix, sig.name).into_token_stream();
    let dtype = utils::gen_type_for_dtype(device, &sig.dtype);
    let optional = sig.optional;

    // the struct filler
    match &sig.dtype {
        DType::UInt { .. }
        | DType::SInt { .. }
        | DType::Buf { .. }
        | DType::Float { .. }
        | DType::Bitset { .. }
        | DType::Enum { .. } => Some(gen_assignment(
            name,
            sig,
            gen_sig_bit_load(sig, dtype.unwrap(), idx),
            *idx,
            check_bounds,
        )),
        DType::None => None,
        DType::Pad { width } => {
            // optional pad is not supported. tf you are doing????
            *idx += *width;
            None
        }
        DType::Bool { default_value } => {
            let pos = *idx;
            *idx += 1;
            // TODO: how 2 do this lol
            Some(gen_assignment(
                name,
                sig,
                quote!(unsafe{*bits.get_unchecked(#pos)}),
                *idx,
                check_bounds,
            ))
        }
        DType::Struct { meta } => {
            // optional structs also not supported. lol
            let mut declrs: Vec<TokenStream> = Vec::new();
            let fields: Vec<TokenStream> = meta
                .signals
                .iter()
                .filter_map(|subsig| {
                    // each sig
                    gen_signal_unpacker(
                        device,
                        subsig,
                        format!("{}_{}", prefix, sig.name),
                        idx,
                        check_bounds,
                    )
                    .map(|(declr, expr_name, struct_fill)| {
                        // append the declr to declrs as a side effect
                        declrs.push(declr);
                        quote!(#expr_name: #struct_fill)
                    })
                })
                .collect();
            let expr_name = format_ident!("{}", sig.name.to_owned());
            let dtype_unwrap = dtype.unwrap();
            Some((
                quote! {
                    #(#declrs)*
                }, // the declarations
                expr_name,
                quote! {
                    #dtype_unwrap {
                        #(#fields),*
                    }
                }, // the fields of the struct that get appended on
            ))
        }
    }
}

pub fn gen_inbound_message_impl(device: &Device, target_source: Source) -> TokenStream {
    let arms: Vec<TokenStream> = device
        .messages
        .iter()
        .map(|(name, msg)| {
            let msg_can_id_first = utils::gen_can_id(device, msg.id);
            let msg_can_id_last = msg_can_id_first + 63;
            let msg_min_dlc = msg.min_length as usize;
            let msg_max_dlc = msg.max_length as usize;
            let msg_name = utils::screaming_snake_to_ident(name);
            let msg_size = if msg_max_dlc != msg_min_dlc {
                quote!(#msg_min_dlc..=#msg_max_dlc)
            } else {
                quote!(#msg_max_dlc)
            };

            let mut idx = 0usize;
            let mut declrs: Vec<TokenStream> = Vec::new();
            let fields: Vec<TokenStream> = msg
                .signals
                .iter()
                .filter_map(|sig| {
                    gen_signal_unpacker(device, sig, "sig".to_string(), &mut idx, false).map(
                        |(sig_declrs, sig_expr_name, sig_struct_fill)| {
                            declrs.push(sig_declrs);
                            quote!(#sig_expr_name: #sig_struct_fill)
                        },
                    )
                })
                .collect();

            // the match arm for a message.
            quote! {
                (#msg_size, #msg_can_id_first..=#msg_can_id_last) => {
                    #(#declrs)*
                    Ok(Message::#msg_name {
                        #(#fields),*
                    })
                }
            }
        })
        .collect();

    let id = if device.dev_type == 31 {
        quote!((cmsg.get_id() & 0xffffff))
    } else {
        quote!(cmsg.get_id())
    };

    // note: this may fuck up if there's no actual messages.
    // if you have a syntax error maybe it's this
    quote! {
        impl<T> TryFrom<&crate::CanandMessageWrapper<T>> for Message where T : crate::CanandMessage<T> {
            type Error = ();
            fn try_from(cmsg: &crate::CanandMessageWrapper<T>) -> Result<Self, Self::Error> {
                use bitvec::prelude::*;
                let dlc = cmsg.get_len() as usize;
                let data = cmsg.get_data();
                let bits = BitSlice::<_, Lsb0>::from_slice(data);

                match (dlc, #id) {
                    #(#arms)*
                    _ => Err(())
                }
            }
        }

        impl<T> TryFrom<crate::CanandMessageWrapper<T>> for Message where T : crate::CanandMessage<T> {
            type Error = ();
            fn try_from(cmsg: crate::CanandMessageWrapper<T>) -> Result<Self, Self::Error> {
                use bitvec::prelude::*;
                let dlc = cmsg.get_len() as usize;
                let data = cmsg.get_data();
                let bits = BitSlice::<_, Lsb0>::from_slice(data);

                match (dlc, #id) {
                    #(#arms)*
                    _ => Err(())
                }
            }
        }

    }
}

// ======================================================================================================

fn gen_sig_bit_store(device: &Device, sig: &Signal, idx: &mut usize) -> TokenStream {
    let width = sig.dtype.bit_length();
    let backing_integral = match sig.dtype {
        DType::SInt { meta: _ } => utils::i_with_size(width),
        _ => utils::u_with_size(width),
    };

    // each type this function handles can either be addressed as a slice or as an integral type (usually unsigned.)
    // which one is used depends on if the signal (and value) is byte-aligned or not.
    let (to_slice, to_integral) = match sig.dtype {
        DType::UInt { meta: _ } => (quote!(&_value.to_le_bytes()), quote!(_value)),
        DType::SInt { meta: _ } => (quote!(&_value.to_le_bytes()), quote!(_value)),
        DType::Float { meta } => match meta.width {
            32 | 64 => (
                quote!(&_value.to_bits().to_le_bytes()),
                quote!(&_value.to_bits()),
            ),
            24 => (
                quote!(&_value.to_bits().to_le_bytes()[1..4]),
                quote!(&(_value.to_bits() >> 8)),
            ),
            _ => panic!("unsupported float width"),
        },
        DType::Buf { meta: _ } => (
            quote!(&_value[..]),
            quote!(#backing_integral::from_le_bytes(_value)),
        ),
        DType::Enum { meta: _ } => (
            quote!(&(_value as #backing_integral).to_le_bytes()),
            quote!(_value as #backing_integral),
        ),
        DType::Bitset { meta: _ } => {
            let arb_ubits = format_ident!("u{}", width);
            (
                quote!(&_value.value().to_le_bytes()),
                quote!(_value.value().value()),
            )
        }
        _ => unreachable!("unsupported dtype passed into gen_sig_bit_store"),
    };
    let (start, end) = (*idx, *idx + width);
    // increment the idx ctr here.
    *idx += width;

    if utils::byte_aligned(width) && (start % 8) == 0 {
        // if both the width is aligned and the starting point is byte aligned, we just do a byte copy (happy path).
        // this uses the to_slice expression
        let start_byte = start / 8;
        let end_byte = end / 8;
        quote! { unsafe { msg_buf.as_raw_mut_slice()[#start_byte..#end_byte].copy_from_slice(#to_slice); } }
    } else {
        // otherwise we convert to some sort of integral type and store it that way
        quote! { unsafe { msg_buf.get_unchecked_mut(#start..#end).store_le::<#backing_integral>(#to_integral); } }
    }
}

pub fn gen_signal_packer(
    device: &Device,
    sig: &Signal,
    prefix: Option<TokenStream>,
    idx: &mut usize,
) -> TokenStream {
    let name = format_ident!("{}", sig.name);

    let qual_name = match prefix {
        Some(prefix_tok) => quote! {#prefix_tok.#name},
        None => quote! {#name},
    };
    let serialize_op = match &sig.dtype {
        DType::UInt { .. }
        | DType::SInt { .. }
        | DType::Buf { .. }
        | DType::Float { .. }
        | DType::Bitset { .. }
        | DType::Enum { .. } => gen_sig_bit_store(device, sig, idx),
        DType::None => quote!(),
        DType::Pad { width } => {
            *idx += width;
            return quote!();
        }
        DType::Bool { default_value } => {
            let start = *idx;
            *idx += 1;
            quote! {unsafe { msg_buf.set_unchecked(#start, _value)}}
        }
        DType::Struct { meta } => utils::flatten_token_vec(
            meta.signals
                .iter()
                .map(|sig| gen_signal_packer(device, sig, Some(qual_name.clone()), idx))
                .collect(),
        ),
    };

    if sig.optional {
        let sig_bytes = (sig.dtype.bit_length() + 7) / 8;
        quote! {
            match #qual_name {
                Some(_value) => {
                    msg_dlc += #sig_bytes;
                    #serialize_op
                },
                None => ()
            }
        }
    } else {
        quote! {
            let _value = #qual_name;
            #serialize_op
        }
    }
}

pub fn gen_outbound_message_impl(device: &Device, target_source: Source) -> TokenStream {
    let device_lname = format_ident!("{}", device.name.to_lowercase());
    let arms: Vec<TokenStream> = device
        .messages
        .iter()
        .map(|(name, msg)| {
            let msg_name = utils::screaming_snake_to_ident(name);
            let sig_names: Vec<TokenStream> = msg.signals.iter().filter_map(|sig| {
                if sig.dtype.is_pad() {
                    None
                } else {
                    Some(format_ident!("{}", sig.name).into_token_stream())
                }
            }).collect();

            let msg_id = utils::gen_can_id(device, msg.id);


            let msg_len = (msg.max_length * 8) as usize;
            let msg_dlc = msg.min_length as usize;
            let mut idx = 0;
            
            let packers : Vec<TokenStream> = msg.signals.iter().map(|sig| {
                gen_signal_packer(device, sig, None, &mut idx)
            }).collect();

            quote! {
                Message::#msg_name { #(#sig_names),* } => {
                    let mut msg_buf: bitvec::BitArr!(for #msg_len, in u8, bitvec::prelude::Lsb0) = BitArray::ZERO;
                    let mut msg_dlc = #msg_dlc;
                    #(#packers)*

                    Ok(crate::CanandMessageWrapper(T::try_from_data(#msg_id | can_device_id, &msg_buf.as_raw_slice()[0..msg_dlc])?))
                }
            }
        }).collect();

    quote! {
        impl CanandDeviceMessage for Message {
            type Index = crate::#device_lname::MessageIndex;

            fn try_into_wrapper<T: crate::CanandMessage<T>>(&self, can_device_id: u32) -> Result<crate::CanandMessageWrapper<T>, crate::CanandMessageError> {
                use bitvec::prelude::*;
                match *self {
                    #(#arms),*
                    _ => core::unreachable!(),
                }
            }

            fn try_from_wrapper<T: crate::CanandMessage<T>>(cmsg: &crate::CanandMessageWrapper<T>) -> Result<Self, ()> {
                cmsg.try_into()
            }
        }
    }
}

pub fn gen_message_index_enum(device: &Device) -> TokenStream {
    let ents: Vec<TokenStream> = device
        .messages
        .iter()
        .map(|(name, msg)| {
            let msg_name = utils::screaming_snake_to_ident(name);
            let msg_id = Literal::u8_unsuffixed(msg.id);
            quote! { #msg_name = #msg_id, }
        })
        .collect();

    let assoc: Vec<TokenStream> = device
        .messages
        .iter()
        .map(|(name, msg)| {
            let msg_name = utils::screaming_snake_to_ident(name);
            let msg_id = Literal::u8_unsuffixed(msg.id);
            quote! { #msg_id => Ok(MessageIndex::#msg_name), }
        })
        .collect();

    quote! {
        #[cfg_attr(feature="device",derive(defmt::Format))]
        #[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
        #[repr(u8)]
        pub enum MessageIndex {
            #(#ents)*
        }

        impl TryFrom<u8> for MessageIndex {
            type Error = ();
            fn try_from(v: u8) -> Result<Self, Self::Error> {
                match v {
                    #(#assoc)*
                    _ => Err(())
                }
            }
        }

        impl From<MessageIndex> for u8 {
            fn from(v: MessageIndex) -> u8 {
                v as u8
            }
        }
    }
}

pub fn gen_message_filters(device: &Device) -> TokenStream {
    let filter_expects: Vec<TokenStream> = device
        .messages
        .iter()
        .map(|(name, msg)| {
            let msg_name = utils::screaming_snake_to_ident(name);
            let filter_numer: u32 = utils::gen_can_id(device, msg.id);
            quote! { #msg_name => #filter_numer, }
        })
        .collect();

    let device_expect = utils::gen_can_id(device, 0);
    quote! {
        pub fn can_filter_for(device_id: u8) -> crate::generic::CanMaskFilter {
            crate::generic::CanMaskFilter {
                expect: #device_expect | device_id as u32,
                mask: 0x1FFF003F
            }
        }

        impl MessageIndex {
            pub fn filter_for(&self, device_id : u8) -> crate::generic::CanMaskFilter {
                crate::generic::CanMaskFilter {
                    expect: device_id as u32 | match self {
                        #(#filter_expects)*
                    },
                    mask: 0x1FFFFFFF
                }
            }
        }
    }
}
