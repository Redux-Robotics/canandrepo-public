use canandmessage_parser::toml_defs::TypeSpec;
use canandmessage_parser::{DType, Device, Setting, Signal, Source, StructMeta};
use proc_macro2::{Literal, TokenStream};
use quote::{format_ident, quote, ToTokens};

use crate::setting_generation::gen_default_settings_value;
use crate::utils::{
    self, f_with_size, flatten_token_vec, gen_type_for_dtype, i_with_size, min_width, u8_buf,
    u_with_size,
};

// TODO: figure out how to serialize/deserialize to the byte format.

pub fn gen_simulation(device: &Device) -> proc_macro2::TokenStream {
    if cfg!(not(feture = "simulation")) {
        //return quote!();
    }
    let type_name = format_ident!(
        "Sim{}",
        crate::utils::screaming_snake_to_camel(&crate::utils::capitalize(device.name.as_str()))
    );
    let lowercase_name = format_ident!("{}", device.name.as_str().to_lowercase());
    let mut type_contents: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut defaults_contents: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut periodic_contents: Vec<proc_macro2::TokenStream> = Vec::new();

    for message in device.messages.iter() {
        if message.1.source == Source::Device {
            if (message.0 == "REPORT_SETTING") {
                periodic_contents.push(quote!(if self.report_instant.elapsed()
                    > std::time::Duration::from_millis(15)
                {
                    match self.settings.report_queue.pop_front() {
                        Some(msg) => {
                            message_buf.push(msg);
                        }
                        _ => {}
                    }
                    self.report_instant = std::time::Instant::now();
                }));
                continue;
            }
            let mut contents_lcl: Vec<proc_macro2::TokenStream> = message
                .1
                .signals
                .iter()
                .filter_map(|sig| {
                    let name = format_ident!(
                        "{}_{}",
                        utils::screaming_snake_to_camel(message.0),
                        sig.name
                    );
                    match gen_type_for_dtype(device, &sig.dtype) {
                        Some(btype) => {
                            if btype.is_empty() {
                                None
                            } else {
                                Some(quote!(
                                    pub #name: #btype
                                ))
                            }
                        }
                        _ => None,
                    }
                })
                .collect();

            type_contents.append(&mut contents_lcl);

            let msg_rate = format_ident!("{}_rate", utils::screaming_snake_to_camel(message.0));

            type_contents.push(quote!(
                pub #msg_rate: std::time::Duration
            ));

            let msg_instant = format_ident!("last_{}", utils::screaming_snake_to_camel(message.0));
            type_contents.push(quote!(
                #msg_instant: std::time::Instant
            ));

            let mut defaults_lcl: Vec<proc_macro2::TokenStream> = message
                .1
                .signals
                .iter()
                .filter_map(|sig| {
                    let name = format_ident!(
                        "{}_{}",
                        utils::screaming_snake_to_camel(message.0),
                        sig.name
                    );
                    match gen_default_value(device, &sig.dtype) {
                        Some(default) => Some(quote!(
                            #name: #default
                        )),
                        _ => None,
                    }
                })
                .collect();

            defaults_lcl.push(quote!(
                #msg_rate: Default::default()
            ));

            defaults_lcl.push(quote!(
                #msg_instant: std::time::Instant::now()
            ));

            defaults_contents.append(&mut defaults_lcl);

            let mut process_signals: Vec<proc_macro2::TokenStream> = message
                .1
                .signals
                .iter()
                .filter_map(|sig| {
                    let sig_name = format_ident!("{}", sig.name);
                    let global_name = format_ident!(
                        "{}_{}",
                        utils::screaming_snake_to_camel(message.0),
                        sig.name
                    );
                    match sig.dtype {
                        DType::Pad { width } => None,
                        _ => Some(quote!(
                            #sig_name: self.#global_name
                        )),
                    }
                })
                .collect();

            let msg_name = format_ident!("{}", utils::screaming_snake_to_camel(message.0));

            if !process_signals.is_empty() {
                periodic_contents.push(quote! (
                    if self.#msg_instant.elapsed() >= self.#msg_rate && self.#msg_rate.as_millis() != 0 {
                        message_buf.push(
                            #lowercase_name::Message::#msg_name {
                                #(#process_signals),*
                            }
                        );

                        self.#msg_instant = std::time::Instant::now();
                    }
                ));
            }
        }

        let mut process_signals: Vec<proc_macro2::TokenStream> = message
            .1
            .signals
            .iter()
            .filter_map(|sig| {
                let name = format_ident!("{}", sig.name);
                match sig.dtype {
                    DType::Pad { width } => None,
                    _ => Some(quote!(
                        #name
                    )),
                }
            })
            .collect();

        let mut process_signal_assign: Vec<proc_macro2::TokenStream> = message
            .1
            .signals
            .iter()
            .filter_map(|sig| match sig.dtype {
                DType::Pad { width } => None,
                _ => {
                    let name = format_ident!("{}", sig.name);
                    let self_name = format_ident!(
                        "{}_{}",
                        utils::screaming_snake_to_camel(message.0),
                        sig.name
                    );
                    Some(quote!(
                        self.#self_name = #name
                    ))
                }
            })
            .collect();
    }

    let num_settings = device
        .settings
        .iter()
        .filter_map(|setting| if setting.1.readable { Some(0) } else { None })
        .collect::<Vec<u8>>()
        .len();

    let mut settings_contents: Vec<proc_macro2::TokenStream> = (device
        .settings
        .iter()
        .filter_map(|setting| {
            if !setting.1.readable {
                return None;
            }

            let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));
            let setting_type = match utils::gen_type_for_dtype(device, &setting.1.dtype) {
                Some(token) => token,
                None => {
                    return None;
                }
            };

            Some(quote!(
                pub #name: #setting_type
            ))
        })
        .collect());

    let mut settings_process_contents: Vec<proc_macro2::TokenStream> = device
        .settings
        .iter()
        .filter_map(|setting| {
            if !setting.1.readable {
                return None;
            }

            let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));

            Some(quote!(
                #lowercase_name::Setting::#name ( value ) => {
                    self.#name = value;
                    self.report_queue.push_back(#lowercase_name::Message::ReportSetting {
                        address: #lowercase_name::types::Setting::#name,
                        value: #lowercase_name::Setting::#name ( self.#name ).into(),
                        flags: #lowercase_name::types::SettingReportFlags::new(true, true)
                    });
                }
            ))
        })
        .collect();

    let mut settings_get_contents: Vec<proc_macro2::TokenStream> = device
        .settings
        .iter()
        .filter_map(|setting| {
            if !setting.1.readable {
                return None;
            }

            let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));

            Some(quote!(
                #lowercase_name::types::Setting::#name => {
                    self.report_queue.push_back(#lowercase_name::Message::ReportSetting {
                        address: #lowercase_name::types::Setting::#name,
                        value: #lowercase_name::Setting::#name ( self.#name ).into(),
                        flags: #lowercase_name::types::SettingReportFlags::new(true, true)
                    });
                }
            ))
        })
        .collect();

    let mut settings_default_content: Vec<proc_macro2::TokenStream> = device
        .settings
        .iter()
        .filter_map(|setting| {
            if !setting.1.readable {
                return None;
            }

            let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));
            let default_val =
                crate::setting_generation::gen_default_settings_value(device, &setting.1.dtype);

            if default_val.is_empty() {
                let true_default = format_ident!("Default::default()");
                return Some(quote!(
                    #name: #true_default
                ));
            }

            Some(quote!(
                #name: #default_val
            ))
        })
        .collect();

    let settings_name = format_ident!(
        "Sim{}Settings",
        crate::utils::screaming_snake_to_camel(&crate::utils::capitalize(device.name.as_str()))
    );

    type_contents.push(quote!(
        pub settings: #settings_name
    ));

    type_contents.push(quote!(
        report_instant: std::time::Instant
    ));

    defaults_contents.push(quote!(
        report_instant: std::time::Instant::now()
    ));

    let report_all_report_queue = gen_report_settings(device);

    quote!(
        #[cfg(feature="simulation")]
        pub struct #type_name {
            #(#type_contents),*
        }

        #[cfg(feature="simulation")]
        impl Default for #type_name {
            fn default() -> #type_name {
                #type_name {
                    #(#defaults_contents),*,
                    settings: Default::default()
                }
            }
        }

        #[cfg(feature="simulation")]
        impl #type_name {
            pub fn sim_periodic(&mut self) -> Vec<#lowercase_name::Message> {
                let mut message_buf: Vec<#lowercase_name::Message> = Vec::new();

                #(#periodic_contents)*

                return message_buf;
            }
        }

        #[cfg(feature="simulation")]
        pub struct #settings_name {
            #(#settings_contents),*,
            pub report_queue: std::collections::VecDeque<#lowercase_name::Message>
        }

        #[cfg(feature="simulation")]
        impl Default for #settings_name {
            fn default() -> #settings_name {
                #settings_name {
                    #(#settings_default_content),*,
                    report_queue: std::collections::VecDeque::new()
                }
            }
        }

        #[cfg(feature="simulation")]
        impl #settings_name {
            pub fn process(&mut self, setting_type: #lowercase_name::types::Setting, setting: #lowercase_name::Setting) {
                match setting {
                    #(#settings_process_contents),*
                    _ => {}
                }
            }

            pub fn add_all_to_report_queue(&mut self) {
                #report_all_report_queue
            }

            pub fn report_setting(&mut self, setting: #lowercase_name::types::Setting) {
                match setting {
                    #(#settings_get_contents),*
                    _ => {}
                }
            }
        }
    )
}

pub fn gen_default_value(dev: &Device, dtype: &DType) -> Option<TokenStream> {
    match dtype {
        DType::None => unreachable!("AAAAAAAAAAAAAAAA HOW DID THIS HAPPEN"),
        DType::UInt { meta } => Some(utils::uint_literal(meta.default_value, meta.width)),
        DType::SInt { meta } => Some(utils::sint_literal(meta.default_value, meta.width)),
        DType::Buf { meta } => Some(utils::buf_literal(&meta.default_value, meta.width)),
        DType::Float { meta } => Some(utils::float_literal(meta.default_value, meta.width)),
        DType::Bitset { meta } => {
            let dtype_name = utils::gen_type_for_dtype(dev, dtype).unwrap();
            let val = utils::uint_literal(meta.default_u64(), meta.width);
            //let arb_un = format_ident!("u{}", meta.width);
            Some(quote!(#dtype_name::from_bitfield(#val)))
        }
        DType::Pad { width } => None,
        DType::Bool { default_value } => Some(quote!(#default_value)),
        DType::Enum { meta } => {
            let dtype_name =
                utils::gen_type_for_dtype(dev, dtype).expect("this should be unreachable");
            if meta.default_value.is_empty() {
                return None;
            }
            let enum_name = utils::screaming_snake_to_ident(&meta.default_value);
            Some(quote!(#dtype_name::#enum_name))
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
                    let value = gen_default_value(dev, &sig.dtype);
                    Some(quote! {
                        #name: #value,
                    })
                })
                .collect();
            Some(quote! {
                #dtype_name {
                    #(#fields)*
                }
            })
        }
    }
}

fn gen_report_settings(dev: &Device) -> TokenStream {
    let lowercase_name = format_ident!("{}", dev.name.as_str().to_lowercase());

    let mut message_assignment: Vec<proc_macro2::TokenStream> = (dev
        .settings
        .iter()
        .filter_map(|setting| {
            if !setting.1.readable {
                return None;
            }

            let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));

            Some(quote!(
                self.report_queue.push_back(
                    #lowercase_name::Message::ReportSetting {
                        address: #lowercase_name::types::Setting::#name,
                        value: #lowercase_name::Setting::#name ( self.#name ).into(),
                        flags: #lowercase_name::types::SettingReportFlags::new(true, true)
                    }
                );
            ))
        })
        .collect());

    return quote!(
        #(#message_assignment)*
    );
}

pub fn gen_simulation_util(devices: &Vec<Device>) -> TokenStream {
    let mut simulation_utils: Vec<TokenStream> = Vec::new();

    for device in devices.iter() {
        simulation_utils.push(gen_simulation(device));
    }

    return quote!(#(#simulation_utils)*);
}
