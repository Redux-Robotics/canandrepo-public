#![allow(warnings)]
use alchemist_generation::gen_alchemist_util;
use canandmessage_parser::Device;
use darling::ast::NestedMeta;
use darling::{Error, FromMeta};
use proc_macro::TokenStream;
use quote::ToTokens;
use std::path::Path;

use quote::quote;
use syn::Data::Struct;
use syn::DataStruct;
use syn::Fields::Named;
use syn::FieldsNamed;
use syn::{parse_macro_input, DeriveInput};

mod alchemist_generation;
mod bitset_generation;
mod device_generation;
mod enum_generation;
mod message_generation;
mod setting_generation;
mod simulation_generation;
mod struct_generation;
mod utils;

#[derive(Debug, FromMeta)]
struct MacroArgs {
    src_file: darling::util::SpannedValue<String>,
    mode: darling::util::SpannedValue<String>,
}

#[derive(Debug, FromMeta)]
struct AlchemistMacroArgs {
    #[darling(multiple)]
    src_file: Vec<String>,
}

#[derive(Debug, FromMeta)]
struct FifoRestMacroArgs {
    #[darling(multiple)]
    src_file: Vec<String>,
}

#[derive(Debug, FromMeta)]
struct SimulationMacroArgs {
    #[darling(multiple)]
    src_file: Vec<String>,
}

/// Proc macros suck. That's just a fact of life.
///
/// Dealing with tokens puts Rust a couple inches ahead of the preprocessor/header hell that is C and C++, but expansion
/// tooling still lags, and writing proc macro code is a great way to watch your processor struggle rerunning
/// cargo check over and over.
/// 
/// At least you can use proc macros in cross compilation contexts.
#[proc_macro_attribute]
pub fn gen_device_messages(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let mut input = syn::parse_macro_input!(input as syn::ItemMod);

    let args = match MacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let src_file =
        Path::new(&std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join(&*args.src_file);
    let device: Device = match canandmessage_parser::parse_spec(&src_file) {
        Ok(v) => v.into(),
        Err(e) => {
            return TokenStream::from(
                darling::Error::custom(e.to_string())
                    .with_span(&args.src_file.span())
                    .write_errors(),
            );
        }
    };
    let mut new_content: Vec<syn::Item> = vec![];
    device_generation::gen_device(&device, (&*args.mode).into(), &mut new_content);
    input.content.as_mut().unwrap().1.append(&mut new_content);
    TokenStream::from(input.to_token_stream())
}

/// I honestly kinda hate this since it will make debugging Alchemist a little bit of hell
/// But fundamentally, the core alchemist utils do depend on the TOML specs
/// Can't wait to do canandmessage typescript edition
#[proc_macro_attribute]
pub fn gen_alchemist_utils(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let mut input = syn::parse_macro_input!(input as syn::ItemMod);

    let args = match AlchemistMacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let mut devices: Vec<Device> = Vec::new();

    let proj_root = std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default();
    for spec in args.src_file.iter() {
        devices.push(
            match canandmessage_parser::parse_spec(&Path::new(&proj_root).join(spec)) {
                Ok(v) => v.into(),
                Err(e) => {
                    return TokenStream::from(
                        darling::Error::custom(e.to_string())
                            .with_span(&spec)
                            .write_errors(),
                    );
                }
            },
        );
    }

    let alchemist_utils: proc_macro2::TokenStream =
        alchemist_generation::gen_alchemist_util(&devices);

    input
        .content
        .as_mut()
        .unwrap()
        .1
        .push(syn::Item::Verbatim(alchemist_utils));

    return TokenStream::from(input.to_token_stream());
}

#[proc_macro_attribute]
pub fn gen_simulation_utils(args: TokenStream, input: TokenStream) -> TokenStream {
    let attr_args = match NestedMeta::parse_meta_list(args.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(Error::from(e).write_errors());
        }
    };
    let mut input = syn::parse_macro_input!(input as syn::ItemMod);

    let args = match SimulationMacroArgs::from_list(&attr_args) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream::from(e.write_errors());
        }
    };

    let mut devices: Vec<Device> = Vec::new();

    let proj_root = std::env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default();
    for spec in args.src_file.iter() {
        devices.push(
            match canandmessage_parser::parse_spec(&Path::new(&proj_root).join(spec)) {
                Ok(v) => v.into(),
                Err(e) => {
                    return TokenStream::from(
                        darling::Error::custom(e.to_string())
                            .with_span(&spec)
                            .write_errors(),
                    );
                }
            },
        );
    }

    let simulation_utils: proc_macro2::TokenStream =
        simulation_generation::gen_simulation_util(&devices);

    input
        .content
        .as_mut()
        .unwrap()
        .1
        .push(syn::Item::Verbatim(simulation_utils));

    return TokenStream::from(input.to_token_stream());
}
