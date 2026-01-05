use crate::alchemist_generation::gen_alchemist;
//use crate::message_generation::{
//    gen_inbound_message_conversions, gen_messages, gen_outbound_message_conversions,
//};
use crate::bitset_generation::{gen_bitsets, gen_faults};
use crate::enum_generation::gen_enums;
use crate::message_generation::{
    gen_inbound_message_impl, gen_message_enum, gen_message_filters, gen_message_index_enum,
    gen_outbound_message_impl,
};
use crate::setting_generation::{
    gen_default_settings_vec, gen_setting_enum,
    gen_setting_enum_pack, gen_setting_enum_unpack,
};
use crate::simulation_generation::gen_simulation;
use crate::struct_generation::gen_structs;
use canandmessage_parser::Device;
use quote::quote;

pub fn gen_device(
    device: &Device,
    tgt_source: canandmessage_parser::Source,
    mod_vec: &mut Vec<syn::Item>,
) {
    let enum_defs = gen_enums(device);
    let struct_defs = gen_structs(device);
    let bitset_defs = gen_bitsets(device);
    let msg_enum = gen_message_enum(device);
    let msg_index = gen_message_index_enum(device);
    let msg_filters = gen_message_filters(device);
    let unpack = gen_inbound_message_impl(device, tgt_source);
    let repack = gen_outbound_message_impl(device, tgt_source.flip());
    let setting_enum = gen_setting_enum(device);
    let setting_enum_unpack = gen_setting_enum_unpack(device);
    let setting_enum_pack = gen_setting_enum_pack(device);
    let setting_default = gen_default_settings_vec(device);
    let faults = gen_faults(device);

    gen_device_info(device, mod_vec);
    mod_vec.push(syn::Item::Verbatim(quote! {
        use crate::traits::*;

        pub mod types {
            use crate::traits::*;
            #bitset_defs
            #enum_defs
            #struct_defs
        }

        #faults

        #msg_enum
        #msg_index
        #msg_filters

        #unpack
        #repack

        #setting_enum
        #setting_enum_unpack
        #setting_enum_pack
        #setting_default
    }))

    // gen_messages(device, mod_vec);
    // gen_inbound_message_conversions(device, &tgt_source, mod_vec);
    // gen_outbound_message_conversions(device, &tgt_source.flip(), mod_vec);
    // gen_settings(device, mod_vec);
}

pub fn gen_device_info(device: &Device, mod_vec: &mut Vec<syn::Item>) {
    use quote::format_ident;
    let dev_name = &device.name;
    let dev_type = device.dev_type;
    let dev_class = device.dev_class;

    let dev_lname = format_ident!("{}", device.name.to_lowercase());

    let dev_info = quote!(
        #[doc="Device Name."]
        pub const DEV_NAME : &str = #dev_name;
        #[doc="Device Type (for purposes of FRC-CAN spec)."]
        pub const DEV_TYPE : u8 = #dev_type;

        #[derive(Debug, Copy, Clone, PartialEq, Eq)]
        pub struct Device;

        impl CanandDevice for Device {
            type Message = crate::#dev_lname::Message;
            type Setting = crate::#dev_lname::Setting;
            const DEV_TYPE: u8 = crate::#dev_lname::DEV_TYPE;
            const DEV_NAME: &'static str = crate::#dev_lname::DEV_NAME;
            fn setting_info<'a>() -> &'a [SettingInfo<Self::Setting>] {
                &crate::#dev_lname::SETTING_INFO
            }
        }
    );
    mod_vec.push(syn::Item::Verbatim(dev_info));
}
