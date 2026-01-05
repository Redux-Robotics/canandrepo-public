use canandmessage_parser::toml_defs::TypeSpec;
use canandmessage_parser::{DType, Device, Setting, Signal, StructMeta};
use darling::FromMeta;
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

use crate::message_generation::{gen_signal_packer, gen_signal_unpacker};
use crate::utils::{
    self, f_with_size, gen_type_for_dtype, i_with_size, min_width, u8_buf, u_with_size,
};

// TODO: figure out how to serialize/deserialize to the byte format.

pub fn gen_setting_enum(device: &Device) -> TokenStream {
    let entries: Vec<TokenStream> = device
        .settings
        .iter()
        .map(|(name, spec)| {
            let ent_name = utils::screaming_snake_to_ident(name);
            let lname = utils::lname(device);
            let dtype = gen_type_for_dtype(device, &spec.dtype).expect(
                "pad/none are invalid types for settings. if the setting has no use, go use buf.",
            );

            let stg_id = utils::uint_literal(spec.id as u64, 8);
            let docstr = spec.comment.as_str();
            quote! {
                #[doc=#docstr]
                #ent_name(#dtype) = #stg_id
            }
        })
        .collect();

    quote! {
        #[cfg_attr(feature="device",derive(defmt::Format))]
        #[cfg_attr(any(feature = "alchemist", feature = "simulation"), derive(serde::Serialize, serde::Deserialize))]
        #[repr(u8)]
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub enum Setting {
            #(#entries),*
        }
    }
}

pub fn gen_setting_enum_unpack(device: &Device) -> TokenStream {
    // really stupid check but whatever
    if !device.messages.contains_key("SET_SETTING") {
        return quote!();
    }

    let lname = utils::lname(device);
    let arms: Vec<TokenStream> = device
        .settings
        .iter()
        .map(|(name, spec)| {
            let ent_name = utils::screaming_snake_to_ident(name);

            let mut idx = 0usize;
            let (sig_declrs, name, sig_struct_fill) =
                gen_signal_unpacker(device, &spec.into(), "sig".to_string(), &mut idx, true)
                    .expect("settings should not be pad or none");

            quote! {
                crate::#lname::types::Setting::#ent_name => {
                    #sig_declrs
                    Ok(crate::#lname::Setting::#ent_name ( #sig_struct_fill ))
                },
            }
        })
        .collect();

    quote! {
        impl CanandDeviceSetting for crate::#lname::Setting {
            type Index = crate::#lname::types::Setting;
            /// Convert a setting address/data pair to a Setting enum.
            fn from_address_data(address: Self::Index, data: &[u8; 6]) -> Result<Self, ()> {
                use bitvec::prelude::*;
                let bits = BitSlice::<_, Lsb0>::from_slice(data);
                match address {
                    #(#arms)*
                }
            }
        }
    }
}

pub fn gen_setting_enum_pack(device: &Device) -> TokenStream {
    if !device.messages.contains_key("SET_SETTING") {
        return quote!();
    }

    let lname = utils::lname(device);
    let packers: Vec<TokenStream> = device
        .settings
        .iter()
        .map(|(name, spec)| {
            let ent_name = utils::screaming_snake_to_ident(name);
            let mut idx = 0usize;
            let packer = gen_signal_packer(device, &spec.into(), None, &mut idx);
            quote! {
                crate::#lname::Setting::#ent_name ( value ) => {
                    #packer
                }
            }
        })
        .collect();

    quote! {
        impl From<crate::#lname::Setting> for [u8; 6] {
            fn from(stg: crate::#lname::Setting) -> [u8; 6] {
                use bitvec::prelude::*;
                let mut msg_buf: bitvec::BitArr!(for 48, in u8, bitvec::prelude::Lsb0) = BitArray::ZERO;
                let mut msg_dlc = 0usize;

                match stg {
                    #(#packers),*
                }

                msg_buf.as_raw_slice().try_into().unwrap()
            }
        }
    }
}

pub fn gen_default_settings_value(dev: &Device, dtype: &DType) -> TokenStream {
    match dtype {
        DType::None => unreachable!("AAAAAAAAAAAAAAAA HOW DID THIS HAPPEN"),
        DType::UInt { meta } => utils::uint_literal(meta.default_value, meta.width),
        DType::SInt { meta } => utils::sint_literal(meta.default_value, meta.width),
        DType::Buf { meta } => utils::buf_literal(&meta.default_value[0..6], meta.width),
        DType::Float { meta } => utils::float_literal(meta.default_value, meta.width),
        DType::Bitset { meta } => {
            let dtype_name = utils::gen_type_for_dtype(dev, dtype).unwrap();
            let val = utils::uint_literal(meta.default_u64(), meta.width);
            //let arb_un = format_ident!("u{}", meta.width);
            quote!(#dtype_name::from_bitfield(#val))
        }
        DType::Pad { width } => quote!(),
        DType::Bool { default_value } => quote!(#default_value),
        DType::Enum { meta } => {
            let dtype_name =
                utils::gen_type_for_dtype(dev, dtype).expect("this should be unreachable");
            let enum_name = utils::screaming_snake_to_ident(&meta.default_value);
            quote!(#dtype_name::#enum_name)
        }
        DType::Struct { meta } => {
            let dtype_name = crate::utils::gen_type_for_dtype(dev, dtype).unwrap();
            let fields: Vec<TokenStream> = meta
                .signals
                .iter()
                .filter_map(|sig| {
                    if sig.dtype.is_pad() {
                        return None;
                    }
                    let name = format_ident!("{}", sig.name);
                    let value = gen_default_settings_value(dev, &sig.dtype);
                    Some(quote! {
                        #name: #value,
                    })
                })
                .collect();
            quote! {
                #dtype_name {
                    #(#fields)*
                }
            }
        }
    }
}

pub fn gen_default_settings_vec(device: &Device) -> TokenStream {
    if !device.messages.contains_key("SET_SETTING") {
        return quote!();
    }
    let lname = crate::utils::lname(device);
    let stgs: Vec<TokenStream> = device
        .settings
        .iter()
        .map(|(name, stg)| {
            let ent_name = crate::utils::screaming_snake_to_ident(name);
            let value = gen_default_settings_value(device, &stg.dtype);

            let readable = stg.readable;
            let writable = stg.writable;
            let reset_on_default = stg.reset_on_default;

            quote! {
                SettingInfo {
                    readable: #readable,
                    writable: #writable,
                    reset_on_default: #reset_on_default,
                    index: crate::#lname::types::Setting::#ent_name,
                    default_value: crate::#lname::Setting::#ent_name(#value)
                }
            }

            //if stg.readable && stg.writable && stg.reset_on_default {
            //    let value = gen_default_settings_value(device, &stg.dtype);
            //    Some(quote! {
            //        (crate::#lname::types::Setting::#ent_name,
            //        crate::#lname::Setting::#ent_name (#value)
            //    )})
            //} else { None }
        })
        .collect();
    let vlen = Literal::usize_unsuffixed(stgs.len());
    //pub static DEFAULT_SETTINGS: [(crate::#lname::types::Setting, crate::#lname::Setting); #vlen] = [#(#stgs),*];

    quote! {
        #[doc="Array of all default settings for the device."]
        pub static SETTING_INFO: [SettingInfo<crate::#lname::Setting>; #vlen] = [#(#stgs),*];
    }
}
