//use crate::message_generation::{
//    gen_inbound_message_conversions, gen_messages, gen_outbound_message_conversions,
//};
use crate::utils::{flatten_token_vec, screaming_snake_to_ident};
use canandmessage_parser::toml_defs::EnumSpec;
use canandmessage_parser::{Device, EnumEntry, EnumMeta};
use proc_macro2::Literal;
use quote::{format_ident, quote};

pub fn gen_enum(spec: &EnumMeta, dev: &Device) -> proc_macro2::TokenStream {
    let name = screaming_snake_to_ident(&spec.name);
    let repr_type = crate::utils::u_with_size(spec.width);

    if !dev.name.eq_ignore_ascii_case(spec.origin_lname.as_str()) {
        let origin_lname = format_ident!("{}", spec.origin_lname);
        return quote!(
            pub type #name = crate::#origin_lname::types::#name;
        );
    }

    let entries: Vec<proc_macro2::TokenStream> = spec
        .values
        .iter()
        .map(|(idx, ent)| {
            let name = screaming_snake_to_ident(&ent.name);
            let val = Literal::u64_unsuffixed(ent.index);
            let docstr = Literal::string(ent.comment.as_str());
            quote! (
                #[doc = #docstr]
                #name = #val
            )
        })
        .collect();

    let assoc: Vec<proc_macro2::TokenStream> = spec
        .values
        .iter()
        .map(|(idx, ent)| {
            let ename = screaming_snake_to_ident(&ent.name);
            let index = Literal::u64_unsuffixed(*idx);
            quote!(#index => Ok(#name::#ename),)
        })
        .collect();

    let default_block = if spec.default_value.len() > 0 {
        let default_value = screaming_snake_to_ident(&spec.default_value);
        quote! {

            impl Default for #name {
                fn default() -> Self {
                    Self::#default_value
                }
            }

        }
    } else {
        quote!()
    };

    let variant_count = spec.values.len();
    let variants_array: Vec<proc_macro2::TokenStream> = spec
        .values
        .iter()
        .map(|(idx, ent)| {
            let vname = screaming_snake_to_ident(&ent.name);
            quote!(#name::#vname)
        })
        .collect();

    // TODO: gonna leave off the derivations for now to make cargo-expand readable
    // There _are_ macros that purport to make conversions from and to primitives easier -- may be worth investigating.
    //
    quote!(
        #[repr(#repr_type)]
        #[cfg_attr(any(feature = "alchemist", feature = "simulation"), derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature="device",derive(defmt::Format))]
        #[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
        pub enum #name {
            #(#entries),*
        }

        impl TryFrom<#repr_type> for #name {
            type Error = ();
            fn try_from(v: #repr_type) -> Result<Self, Self::Error> {
                match v {
                    #(#assoc)*
                    _ => Err(())
                }
            }
        }

        impl From<#name> for #repr_type {
            fn from(v: #name) -> #repr_type {
                v as #repr_type
            }
        }
        #default_block
        impl #name {
            pub const fn variants() -> &'static [Self] {
                const VARIANTS: [#name; #variant_count] = [#(#variants_array),*];
                &VARIANTS
            }
        }
    )
}

pub fn gen_enums(device: &Device) -> proc_macro2::TokenStream {
    flatten_token_vec(
        device
            .enums
            .iter()
            .map(|(name, spec)| gen_enum(spec, device))
            .collect(),
    )
}
