use std::cmp::min;

// Bitset definitions go here.
use canandmessage_parser::toml_defs::TypeSpec;
use canandmessage_parser::{BitsetMeta, DType, Device, Signal, StructMeta};
use proc_macro2::Literal;
use quote::{format_ident, quote, ToTokens};

use crate::utils::{
    f_with_size, flatten_token_vec, i_with_size, min_width, u8_buf, u_with_size, uint_literal,
};

// TODO: figure out how to serialize/deserialize to the byte format.

pub fn gen_bitset(name: &String, spec: &BitsetMeta, dev: &Device) -> proc_macro2::TokenStream {
    let type_name_str =
        crate::utils::screaming_snake_to_camel(&crate::utils::capitalize(name.as_str()));
    let type_name = format_ident!("{type_name_str}");
    let type_name_literal = Literal::string(type_name_str.as_str());

    if !dev.name.eq_ignore_ascii_case(spec.origin_lname.as_str()) {
        let origin_lname = format_ident!("{}", spec.origin_lname);
        return quote!(
            pub type #type_name = crate::#origin_lname::types::#type_name;
        );
    }

    let width = spec.width;
    let u32width = width as u32;
    let container = u_with_size(width);

    let type_contents: Vec<proc_macro2::TokenStream> = spec
        .flags
        .iter()
        .map(|ent| {
            let setter = format_ident!("set_{}", ent.name.to_owned());
            let getter = format_ident!("{}", ent.name.to_owned());
            let set_docstr = Literal::string(format!("Sets {}", &ent.comment).as_str());
            let get_docstr = Literal::string(format!("Gets {}", &ent.comment).as_str());
            let idx = ent.bit_idx;
            quote!(
                #[doc=#get_docstr]
                pub fn #getter(&self) -> bool {
                    self.get_index(#idx)
                }

                #[doc=#set_docstr]
                pub fn #setter(&mut self, value: bool) {
                    self.set_index(#idx, value)
                }
            )
        })
        .chain(((spec.flags.len())..width).into_iter().map(|idx| {
            let name = format!("reserved_{}", idx);
            let setter = format_ident!("set_{}", &name);
            let getter = format_ident!("{}", &name);
            let set_docstr = Literal::string("Gets a reserved bit");
            let get_docstr = Literal::string("Sets reserved bit");
            let u32idx = idx as u32;
            quote! (
                #[doc=#get_docstr]
                pub fn #getter(mut self) -> bool {
                    self.get_index(#u32idx)
                }

                #[doc=#set_docstr]
                pub fn #setter(&mut self, value: bool) {
                    self.set_index(#u32idx, value)
                }
            )
        }))
        .collect();

    let (constructor_params, constructor_setters): (
        Vec<proc_macro2::TokenStream>,
        Vec<proc_macro2::TokenStream>,
    ) = spec
        .flags
        .iter()
        .map(|ent| {
            let name = format_ident!("{}", &ent.name);
            let setter = format_ident!("set_{}", &ent.name);
            (quote!(#name: bool), quote!(blank.#setter(#name);))
        })
        .unzip();

    let mut debug_fields: Vec<proc_macro2::TokenStream> = spec
        .flags
        .iter()
        .map(|ent| {
            let field_name = Literal::string(&ent.name.as_str());
            let field_value = ent.bit_idx;
            quote!(.field(#field_name, &self.get_index(#field_value)))
        })
        .collect();
    let spec_len = spec.flags.len();
    let spec_min_width = min_width(width);
    if spec.flags.len() < spec_min_width {
        debug_fields.push(quote!(.field("reserved_bits", &(self.0 >> #spec_len))))
    }

    let mut defmt_fields: String = spec
        .flags
        .iter()
        .map(|ent| format!("{}: {{0={}..{}}}", ent.name, ent.bit_idx, ent.bit_idx + 1))
        .chain(
            if spec_len < spec_min_width {
                Some(format!("reserved_bits: {{0={spec_len}..{spec_min_width}}}"))
            } else {
                None
            }
            .into_iter(),
        )
        .reduce(|cur: String, nxt: String| format!("{cur}, {nxt}"))
        .unwrap();

    let container_bit = uint_literal(1, width);
    let defmt_format_string =
        Literal::string(format!("{type_name_str} {{{{ {defmt_fields} }}}}").as_str());

    quote!(
        #[cfg_attr(any(feature = "alchemist", feature = "simulation"), derive(serde::Serialize, serde::Deserialize))]
        #[derive(PartialEq, Eq, Clone, Copy)]
        pub struct #type_name(#container);

        impl #type_name {
            pub const fn from_bitfield(field: #container) -> Self {
                Self(field)
            }
            pub fn new(
                #(#constructor_params),*
            ) -> Self {
                let mut blank = Self(0);
                #(#constructor_setters)*
                blank
            }

            #(#type_contents)*
        }

        impl crate::traits::Bitset<#container> for #type_name {
            fn set_index(&mut self, idx: u32, value: bool) {
                let mask: #container = !((#container_bit).checked_shl(idx).unwrap_or(0));
                self.0 = (self.0 & mask) | (value as #container).checked_shl(idx).unwrap_or(0);
            }

            fn get_index(&self, idx: u32) -> bool {
                (self.0.checked_shr(idx).unwrap_or(0) & 0b1) != 0
            }

            fn value(&self) -> #container {
                self.0
            }
        }

        impl From<#container> for #type_name {
            fn from(value: #container) -> #type_name {
                #type_name::from_bitfield(value)
            }
        }

        impl From<#type_name> for #container {
            fn from(value: #type_name) -> #container {
                value.value()
            }
        }

        impl core::fmt::Debug for #type_name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                //f.debug_tuple("Thingymabob").field(&self.0).finish()
                f.debug_struct(#type_name_literal)
                #(#debug_fields)*
                .finish()
            }
        }

        #[cfg(feature = "device")]
        impl defmt::Format for #type_name {
            fn format(&self, fmt: defmt::Formatter) {
                defmt::write!(fmt, #defmt_format_string, self.0)
            }
        }


    )
}

pub fn gen_bitsets(device: &Device) -> proc_macro2::TokenStream {
    flatten_token_vec(
        device
            .bitsets
            .iter()
            .map(|(name, spec)| gen_bitset(name, spec, device))
            .collect(),
    )
}

pub fn gen_faults(dev: &Device) -> proc_macro2::TokenStream {
    let Some(faults_meta) = dev.bitsets.get("faults") else {
        return quote!();
    };
    let dev_lname = format_ident!("{}", dev.name.to_lowercase());
    let faults_uint = crate::utils::u_with_size(faults_meta.width);

    let contains_power_cycle = faults_meta
        .flags
        .iter()
        .any(|flag| flag.name.as_str() == "power_cycle");

    let set_power_cycle = if contains_power_cycle {
        quote!(boot_sticky_faults.set_power_cycle(true);)
    } else {
        quote!()
    };

    let set_clear_flags: proc_macro2::TokenStream = faults_meta
        .flags
        .iter()
        .map(|flag| {
            let report_flag = format_ident!("report_{}_fault", flag.name);
            let clear_flag = format_ident!("clear_{}_fault", flag.name);
            let set_flag = format_ident!("set_{}", flag.name);
            quote! {
                pub fn #report_flag(&mut self) {
                    self.faults.#set_flag(true);
                    self.sticky_faults.#set_flag(true);
                }

                pub fn #clear_flag(&mut self) {
                    self.faults.#set_flag(false);
                }
            }
        })
        .collect();

    quote! {
        pub struct FaultManager {
            pub faults: crate::#dev_lname::types::Faults,
            pub sticky_faults: crate::#dev_lname::types::Faults,
        }

        impl FaultManager {
            pub fn new() -> Self {
                let boot_faults = crate::#dev_lname::types::Faults::from(0);
                let mut boot_sticky_faults = crate::#dev_lname::types::Faults::from(0);
                #set_power_cycle
                Self { faults: boot_faults, sticky_faults: boot_sticky_faults }
            }

            pub fn get_faults(&self) -> #faults_uint {
                self.faults.value()
            }

            pub fn get_sticky_faults(&self) -> #faults_uint {
                self.sticky_faults.value()
            }

            pub fn clear_sticky_faults(&mut self) {
                self.sticky_faults = self.faults;
            }

            #set_clear_flags
        }
    }
}

/*
 *use canandmessage::canandgyro::types::Faults;

pub struct FaultManager {
    faults : Faults,
    sticky_faults : Faults,
}
impl FaultManager {
    pub fn new() -> Self {
        let boot_faults = Faults::from(0);
        let mut boot_sticky_faults = Faults::from(0);
        boot_sticky_faults.set_power_cycle(true);
        Self {
            faults : boot_faults,
            sticky_faults : boot_sticky_faults,
        }
    }

    pub fn get_faults(&self) -> u8 {
        self.faults.value
    }
    pub fn get_sticky_faults(&self) -> u8 {
        self.sticky_faults.value
    }

    pub fn clear_sticky_faults(&mut self) {
        self.sticky_faults = self.faults;
    }

    pub fn report_power_cycle_fault(&mut self) {
        self.faults.set_power_cycle(true);
        self.sticky_faults.set_power_cycle(true);
    }

    pub fn clear_power_cycle_fault(&mut self) {
        self.faults.set_power_cycle(false);
    }
 *
 *
 */
