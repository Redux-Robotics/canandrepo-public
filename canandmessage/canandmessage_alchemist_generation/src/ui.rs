use canandmessage_parser::{DType, Device};
use darling::{ast::NestedMeta, Error, FromMeta};
use proc_macro::TokenStream;
use quote::format_ident;
use quote::quote;
use quote::ToTokens;
