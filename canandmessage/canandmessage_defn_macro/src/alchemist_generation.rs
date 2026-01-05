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

pub fn gen_alchemist(device: &Device) -> proc_macro2::TokenStream {
    let type_name = format_ident!(
        "{}",
        crate::utils::screaming_snake_to_camel(&crate::utils::capitalize(device.name.as_str()))
    );
    let mut type_contents: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut defaults_contents: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut process_contents: Vec<proc_macro2::TokenStream> = Vec::new();

    let global_name = format_ident!("{}", device.name.as_str().to_lowercase());

    for message in device.messages.iter() {
        if message.1.source == Source::Host {
            continue;
        }

        if (utils::screaming_snake_to_camel(message.0) == "ReportSetting") {
            continue;
        }

        /**
         * Generates the contents of the device class
         *
         * Each signal has the fields that are available marked as SignalName_FieldName
         */
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

        /**
         * Generates the default values contents of the device class
         *
         * We use the defaults indicated in spec if available, otherwise Default::default() or a hardcoded default
         */
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

        defaults_contents.append(&mut defaults_lcl);

        /**
         * This is used further down, this simply represents the name of each item in a signal
         */
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

        /**
         * Used further down, used to manage the assignment of the name of an item in a signal to the global parent object
         *
         * Really this should group together similar objects too, e.g. vel and pos output on a Canandmag both have magnet status
         */
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

        let msg_name = format_ident!("{}", utils::screaming_snake_to_camel(message.0));

        if !process_signals.is_empty()
            && !(utils::screaming_snake_to_camel(message.0) == "ReportSetting")
        {
            process_contents.push(quote!(
                #global_name::Message::#msg_name {
                    #(#process_signals),*
                } => {
                    #(#process_signal_assign);*
                }
            ));
        }
    }

    /**
     * Contents of the variables in a DeviceSettings object
     */
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

    /**
     * This handles settings assignment, for when a report settings CAN packet comes in.
     */
    let mut settings_process_contents: Vec<proc_macro2::TokenStream> = device
        .settings
        .iter()
        .filter_map(|setting| {
            if !setting.1.readable {
                return None;
            }

            let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));

            Some(quote!(
                #global_name::Setting::#name ( value ) => {
                    self.#name = value
                }
            ))
        })
        .collect();

    /**
     * Diff contents:
     * 
     * Settings diff is a method that returns an array of the settings that are different between itself and a passed settings object
     * Super useful for checking if all settings were set
     */
    let mut settings_diff_contents: Vec<proc_macro2::TokenStream> = device.settings.iter().filter_map(|setting| {
        if !setting.1.readable {
            return None;
        }

        let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));

        let lcl_name = utils::screaming_snake_to_camel(setting.0);

        if lcl_name == "CanId" || lcl_name == "SerialNumber" || lcl_name == "Name0" || lcl_name == "Name1" || lcl_name == "Name2" || lcl_name == "FirmwareVersion"{
            return None;
        }
        
        Some(quote!(
            if self.#name != other.#name {
                changed.push((#global_name::types::Setting::#name, #global_name::Setting::#name (other.#name)));
            }
        ))
    }).collect();

    /**
     * Defaults for the settings object. Use spec default, otherwise use Default::default()
     */
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

    let mut settings_differental_contents: Vec<proc_macro2::TokenStream> = device.settings.iter().filter_map(|setting| {
        if !setting.1.readable {
            return None;
        }

        let name = format_ident!("{}", utils::screaming_snake_to_camel(setting.0));

        Some(quote!(
            if other.#name == self.#name {
                diff.push((#global_name::types::Setting::#name, #global_name::Setting::#name (other.#name)));
            }
        ))
    }).collect();

    let settings_name = format_ident!(
        "{}Settings",
        crate::utils::screaming_snake_to_camel(&crate::utils::capitalize(device.name.as_str()))
    );

    type_contents.push(quote!(
        pub settings: #settings_name
    ));

    type_contents.push(quote!(
        #[serde(skip, default = "std::time::Instant::now")]
        pub last_recv: std::time::Instant
    ));

    type_contents.push(quote!(
        pub in_id_conflict: bool
    ));

    quote!(

        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        pub struct #type_name {
            #(#type_contents),*
        }

        impl Default for #type_name {
            fn default() -> #type_name {
                #type_name {
                    #(#defaults_contents),*,
                    settings: Default::default(),
                    last_recv: std::time::Instant::now(),
                    in_id_conflict: false
                }
            }
        }

        impl #type_name {
            pub fn process(&mut self, message: #global_name::Message) {
                self.last_recv = std::time::Instant::now();
                match message {
                    #(#process_contents),*
                    #global_name::Message::ReportSetting { address, value, flags } => {
                        if let Ok(setting_val) = #global_name::Setting::from_address_data(address, &value) {
                            self.settings.process(address, setting_val);
                        }else{
                        }
                    }
                    _ => {}
                };
            }

            pub fn blink_leds_command<T: crate::CanandMessage<T>>(&self, level: u8) -> Option<T> {
                self.to_canandmessage(#global_name::Message::PartyMode { party_level: level })
            }

            pub fn enumerate_command<T: crate::CanandMessage<T>>(&self) -> Option<T> {
                self.to_canandmessage(#global_name::Message::SettingCommand { control_flag: #global_name::types::SettingCommand::FetchSettings, setting_index: None })
            }

            pub fn getserial_command<T: crate::CanandMessage<T>>(&self) -> Option<T> {
                self.to_canandmessage(#global_name::Message::SettingCommand { control_flag: #global_name::types::SettingCommand::FetchSettingValue, setting_index: Some(#global_name::types::Setting::SerialNumber) })
            }


            pub fn change_can_id<T: crate::CanandMessage<T>>(&self, newid: u8) -> Option<T> {
                let message = #global_name::Message::SetSetting { address: #global_name::types::Setting::CanId, value: #global_name::Setting::CanId ( newid ).into(), flags: #global_name::types::SettingFlags { ephemeral: false, synch_hold: false, synch_msg_count: 0} };

                return self.to_canandmessage::<T>(message);
            }

            pub fn set_name_0<T: crate::CanandMessage<T>>(&self, name: [u8; 6]) -> Option<T> {
                let message = #global_name::Message::SetSetting { address: #global_name::types::Setting::Name0, value: #global_name::Setting::Name0 ( name ).into(), flags: #global_name::types::SettingFlags { ephemeral: false, synch_hold: false, synch_msg_count: 0} };

                return self.to_canandmessage::<T>(message);
            }

            pub fn set_name_1<T: crate::CanandMessage<T>>(&self, name: [u8; 6]) -> Option<T> {
                let message = #global_name::Message::SetSetting { address: #global_name::types::Setting::Name1, value: #global_name::Setting::Name1 ( name ).into(), flags: #global_name::types::SettingFlags { ephemeral: false, synch_hold: false, synch_msg_count: 0} };

                return self.to_canandmessage::<T>(message);
            }

            pub fn set_name_2<T: crate::CanandMessage<T>>(&self, name: [u8; 6]) -> Option<T> {
                let message = #global_name::Message::SetSetting { address: #global_name::types::Setting::Name2, value: #global_name::Setting::Name2 ( name ).into(), flags: #global_name::types::SettingFlags { ephemeral: false, synch_hold: false, synch_msg_count: 0} };

                return self.to_canandmessage::<T>(message);
            }

            pub fn clear_sticky_faults<T: crate::CanandMessage<T>>(&self) -> Option<T> {
                let message = #global_name::Message::ClearStickyFaults {};
                return self.to_canandmessage::<T>(message);
            }

            pub fn arbitrate<T: crate::CanandMessage<T>>(&self, id: [u8; 6]) -> Option<T> {
                let message = #global_name::Message::CanIdArbitrate { addr_value: [id[0], id[1], id[2], id[3], id[4], id[5], 0, 0] };
                return self.to_canandmessage::<T>(message);
            }

            pub fn reset_factory_default<T: crate::CanandMessage<T>>(&self) -> Option<T> {
                let message = #global_name::Message::SettingCommand { control_flag: #global_name::types::SettingCommand::ResetFactoryDefault, setting_index: None };
                return self.to_canandmessage::<T>(message);
            }

            fn to_canandmessage<T: crate::CanandMessage<T>>(&self, message: #global_name::Message) -> Option<T> {
                let msg_opt: Option<#global_name::Message> = message.into();

                if let Some(msg) = msg_opt {
                    let canmsg: crate::CanandMessageWrapper<T> = msg.try_into_wrapper(self.settings.CanId.into()).ok()?;
                    let mut canandmessage = canmsg.0;

                    return Some(canandmessage);
                }else{
                    return None;
                }
            }
        }

        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        pub struct #settings_name {
            #(#settings_contents),*
        }

        impl Default for #settings_name {
            fn default() -> #settings_name {
                #settings_name {
                    #(#settings_default_content),*
                }
            }
        }

        impl #settings_name {
            pub fn process(&mut self, setting_type: #global_name::types::Setting, setting: #global_name::Setting) {
                match setting {
                    #(#settings_process_contents),*
                    _ => {}
                }
            }

            pub fn get_changed(&self, other: &#settings_name) -> Vec<(#global_name::types::Setting, #global_name::Setting)> {
                let mut changed: Vec<(#global_name::types::Setting, #global_name::Setting)> = Vec::new();

                #(#settings_diff_contents)*

                return changed;
            }

            pub fn get_name(&self) -> String {
                let mut namearr: [u8; 18] = [0; 18];

                namearr[0] = self.Name0[0];
                namearr[1] = self.Name0[1];
                namearr[2] = self.Name0[2];
                namearr[3] = self.Name0[3];
                namearr[4] = self.Name0[4];
                namearr[5] = self.Name0[5];

                namearr[6] = self.Name1[0];
                namearr[7] = self.Name1[1];
                namearr[8] = self.Name1[2];
                namearr[9] = self.Name1[3];
                namearr[10] = self.Name1[4];
                namearr[11] = self.Name1[5];

                namearr[12] = self.Name2[0];
                namearr[13] = self.Name2[1];
                namearr[14] = self.Name2[2];
                namearr[15] = self.Name2[3];
                namearr[16] = self.Name2[4];
                namearr[17] = self.Name2[5];

                return String::from_utf8(namearr.clone().to_vec()).unwrap();
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

pub fn gen_alchemist_util(devices: &Vec<Device>) -> proc_macro2::TokenStream {
    let mut enum_variants: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut match_contents: Vec<proc_macro2::TokenStream> = Vec::new();
    let mut get_class_contents: Vec<proc_macro2::TokenStream> = Vec::new();

    let mut dev_structs: Vec<proc_macro2::TokenStream> = Vec::new();

    for device in devices.iter() {
        let enum_variant_name = format_ident!("{}", device.name.to_ascii_uppercase());
        let type_name = format_ident!("{}", device.name);
        enum_variants.push(quote!(
            #enum_variant_name {
                device: #type_name
            }
        ));

        match_contents.push(quote!(
            ReduxDevice::#enum_variant_name { device }
        ));

        let num = utils::uint_literal(device.dev_type.into(), 8);

        get_class_contents.push(quote!(
            ReduxDevice::#enum_variant_name { device } => {
                return #num
            }
        ));

        let mut alchemist_defs = gen_alchemist(device);
        //let mut sim_defs = gen_simulation(device);

        dev_structs.push(quote!(
            #alchemist_defs
        ));
    }

    return quote!(
        #(#dev_structs)*

        #[derive(serde::Serialize, serde::Deserialize, Clone)]
        pub enum ReduxDevice {
            #(#enum_variants),*
        }

        impl ReduxDevice {
            pub fn match_serial_number(&self, id: &[u8; 6]) -> bool {
                match self {
                    #( #match_contents => { device.settings.SerialNumber.iter().eq(id.iter()) }),*
                }
            }

            pub fn match_can_id(&self, id: u8) -> bool {
                match self {
                    #( #match_contents => { device.settings.CanId == id }),*
                }
            }

            pub fn get_device_class(&self) -> u8 {
                match self {
                    #( #get_class_contents ),*
                }
            }

            pub fn get_serial_number(&self) -> [u8; 6] {
                match self {
                    #( #match_contents => { device.settings.SerialNumber }),*
                }
            }

            pub fn get_name(&self) -> String {
                match self {
                    #( #match_contents => { device.settings.get_name() }),*
                }
            }

            pub fn get_can_id(&self) -> u8 {
                match self {
                    #( #match_contents => { device.settings.CanId }),*
                }
            }

            pub fn get_last_recv(&self) -> std::time::Instant {
                match self {
                    # ( #match_contents => { device.last_recv }),*
                }
            }

            pub fn change_can_id<T: crate::CanandMessage<T>>(&self, newcanid: u8) -> Option<T> {
                match self {
                    # ( #match_contents => { device.change_can_id(newcanid) }),*
                }
            }

            pub fn blink_leds<T: crate::CanandMessage<T>>(&self, level: u8) -> Option<T> {
                match self {
                    # ( #match_contents => { device.blink_leds_command(level) }),*
                }
            }

            pub fn get_firmware_version(&self) -> Option<String> {
                match self {
                    # ( #match_contents => { Some(format!("{}.{}.{}", device.settings.FirmwareVersion.firmware_year, device.settings.FirmwareVersion.firmware_minor, device.settings.FirmwareVersion.firmware_patch)) }),*
                }
            }

            pub fn set_name_0<T: crate::CanandMessage<T>>(&self, name0: [u8; 6]) -> Option<T> {
                match self {
                    # ( #match_contents => { device.set_name_0(name0) }),*
                }
            }

            pub fn set_name_1<T: crate::CanandMessage<T>>(&self, name0: [u8; 6]) -> Option<T> {
                match self {
                    # ( #match_contents => { device.set_name_1(name0) }),*
                }
            }

            pub fn set_name_2<T: crate::CanandMessage<T>>(&self, name0: [u8; 6]) -> Option<T> {
                match self {
                    # ( #match_contents => { device.set_name_2(name0) }),*
                }
            }

            pub fn clear_sticky_faults<T: crate::CanandMessage<T>>(&self) -> Option<T> {
                match self {
                    # ( #match_contents => { device.clear_sticky_faults() }),*
                }
            }

            pub fn mark_can_id_conflict(&mut self, in_conflict: bool) {
                match self {
                    # ( #match_contents => { device.in_id_conflict = in_conflict; }),*
                }
            }

            pub fn in_id_conflict(&self) -> bool {
                match self {
                    # ( #match_contents => { device.in_id_conflict }),*
                }
            }

            pub fn arbitrate<T: crate::CanandMessage<T>>(&self, id: [u8; 6]) -> Option<T> {
                match self {
                    # ( #match_contents => { device.arbitrate(id) }),*
                }
            }

            pub fn reset_factory_default<T: crate::CanandMessage<T>>(&self) -> Option<T> {
                match self {
                    # ( #match_contents => { device.reset_factory_default() }),*
                }
            }

            pub fn enumerate_command<T: crate::CanandMessage<T>>(&self) -> Option<T> {
                match self {
                    # ( #match_contents => { device.enumerate_command() }),*
                }
            }

            pub fn set_last_recv(&mut self) {
                match self {
                    # ( #match_contents => { device.last_recv = std::time::Instant::now(); }),*
                }
            }
        }
    );
}
