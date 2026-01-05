use canandmessage_parser::toml_defs::TypeSpec;
use canandmessage_parser::{DType, Device, Signal, StructMeta};
use proc_macro2::Literal;
use quote::{format_ident, quote, ToTokens};

use crate::utils::{
    f_with_size, flatten_token_vec, gen_type_for_dtype, i_with_size, min_width, u8_buf, u_with_size,
};

// TODO: figure out how to serialize/deserialize to the byte format.

pub fn gen_struct(device: &Device, name: &String, spec: &StructMeta) -> proc_macro2::TokenStream {
    let type_name = format_ident!(
        "{}",
        crate::utils::screaming_snake_to_camel(&crate::utils::capitalize(name.as_str()))
    );

    if !device.name.eq_ignore_ascii_case(spec.origin_lname.as_str()) {
        let origin_lname = format_ident!("{}", spec.origin_lname);
        return quote!(
            pub type #type_name = crate::#origin_lname::types::#type_name;
        );
    }

    let type_contents: Vec<proc_macro2::TokenStream> = spec
        .signals
        .iter()
        .filter_map(|sig| {
            let name = format_ident!("{}", sig.name);
            let doc_str = Literal::string(sig.comment.as_str());
            match gen_type_for_dtype(device, &sig.dtype) {
                Some(btype) => Some(quote!(
                    #[doc=#doc_str]
                    pub #name: #btype
                )),
                _ => None,
            }
        })
        .collect();
    quote!(
        #[cfg_attr(any(feature= "alchemist", feature = "simulation"),derive(serde::Serialize, serde::Deserialize))]
        #[cfg_attr(feature="device",derive(defmt::Format))]
        #[derive(Debug, PartialEq, Clone, Copy)]
        pub struct #type_name {
            #(#type_contents),*
        }
    )
}

pub fn gen_structs(device: &Device) -> proc_macro2::TokenStream {
    flatten_token_vec(
        device
            .structs
            .iter()
            .map(|(name, spec)| gen_struct(device, name, spec))
            .collect(),
    )
}
