// Data model.
use std::{collections::BTreeMap, path::Path};
use std::{error, fs};
use toml_defs::{DeviceSpec, EnumEntrySpec, EnumSpec};

pub mod model_impl;
pub mod toml_defs;
pub mod utils;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct UIntMeta {
    pub width: usize,
    pub min: Option<u64>,
    pub max: Option<u64>,
    pub default_value: u64,
    pub factor_num: i64,
    pub factor_den: i64,
    // not implemented: offset
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct SIntMeta {
    pub width: usize,
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub default_value: i64,
    pub factor_num: i64,
    pub factor_den: i64,
    // not implemented: offset
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct FloatMeta {
    pub width: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub default_value: f64,
    pub allow_nan_inf: bool,
    pub factor_num: i64,
    pub factor_den: i64,
    // not implemented: offset
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BitsetFlag {
    pub bit_idx: u32,
    pub default_value: bool, // honestly idk lol
    pub name: String,
    pub comment: String,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BufMeta {
    pub width: usize,
    // yeah idk about this one either feel free to replace with Sane data type
    pub default_value: [u8; 8],
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnumMeta {
    pub name: String,
    pub origin_lname: String,
    pub width: usize,
    pub default_value: String, // default "index"
    pub is_public: bool,
    pub values: BTreeMap<u64, EnumEntry>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct EnumEntry {
    pub name: String,
    pub comment: String,
    pub index: u64,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StructMeta {
    pub name: String,
    pub origin_lname: String,
    pub signals: Vec<Signal>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BitsetMeta {
    pub name: String,
    pub origin_lname: String,
    pub width: usize,
    pub flags: Vec<BitsetFlag>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum DType {
    None,
    UInt { meta: UIntMeta },
    SInt { meta: SIntMeta },
    Buf { meta: BufMeta },
    Float { meta: FloatMeta },
    Bitset { meta: BitsetMeta },
    Pad { width: usize },
    Bool { default_value: bool },
    Enum { meta: EnumMeta },
    Struct { meta: StructMeta },
}

#[derive(Debug, PartialEq, Clone)]
pub struct Signal {
    pub name: String,
    pub comment: String,
    pub dtype: DType,
    pub optional: bool,
    // NOT implemented: mux, muxed_by, muxed_match
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Source {
    Device,
    Host,
    Both,
}

#[derive(Debug)]
pub struct Message {
    pub id: u8,
    pub comment: String,
    pub max_length: u8,
    pub min_length: u8,
    pub source: Source,
    pub is_public: bool,
    pub signals: Vec<Signal>,
    pub origin_lname: String,
}

#[derive(Debug)]
pub struct Setting {
    pub name: String,
    pub id: u8,
    pub comment: String,
    pub dtype: DType,
    pub vendordep: bool,
    pub vdep_setting: bool,
    pub readable: bool,
    pub writable: bool,
    pub reset_on_default: bool,
    pub special_flags: Vec<String>,
    pub origin_lname: String,
}
#[derive(Debug)]
pub struct Device {
    pub name: String,
    pub arch: String,
    pub dev_type: u8,
    pub dev_class: u8,
    pub messages: BTreeMap<String, Message>,
    pub settings: BTreeMap<String, Setting>,
    pub enums: BTreeMap<String, EnumMeta>,
    pub structs: BTreeMap<String, StructMeta>,
    pub bitsets: BTreeMap<String, BitsetMeta>,
    pub java_package: String,
    pub cpp_namespace: String,
}

fn regen_settings_enum(spec: &DeviceSpec) -> EnumSpec {
    EnumSpec {
        btype: "uint".to_string().to_owned(),
        bits: 8,
        is_public: true,
        default_value: "".to_string().to_owned(),
        values: spec
            .settings
            .iter()
            .map(|(name, stg)| {
                (
                    name.to_owned(),
                    EnumEntrySpec {
                        id: stg.id as u32,
                        comment: stg.comment.to_owned(),
                    },
                )
            })
            .collect(),
        origin_lname: spec.name.to_lowercase(),
    }
}

fn regen_setting_commands_enum(spec: &DeviceSpec) -> EnumSpec {
    EnumSpec {
        btype: "uint".to_string().to_owned(),
        bits: 8,
        is_public: true,
        default_value: "".to_string().to_owned(),
        values: spec
            .setting_commands
            .iter()
            .map(|(name, stg)| {
                (
                    name.to_owned(),
                    EnumEntrySpec {
                        id: stg.id as u32,
                        comment: stg.comment.to_owned(),
                    },
                )
            })
            .collect(),
        origin_lname: spec.name.to_lowercase(),
    }
}

fn assign_origins(spec: &mut DeviceSpec) {
    for enum_ in spec.enums.iter_mut() {
        if enum_.1.origin_lname.is_empty() {
            enum_.1.origin_lname = spec.name.to_lowercase();
        }
    }
    for type_ in spec.types.iter_mut() {
        if type_.1.origin_lname.is_empty() {
            type_.1.origin_lname = spec.name.to_lowercase();
        }
    }
}

pub fn parse_spec(spec_path: &Path) -> Result<DeviceSpec, Box<dyn error::Error>> {
    let toml_str: String = fs::read_to_string(spec_path)?;
    let mut dev_spec: DeviceSpec = toml::from_str(&toml_str)?;
    assign_origins(&mut dev_spec);
    let dev: DeviceSpec = if dev_spec.base.len() > 0 {
        dev_spec.base.clone().iter().fold(
            Ok(dev_spec),
            |a: Result<DeviceSpec, Box<dyn error::Error>>, v| {
                let mut base_spec = parse_spec(
                    &spec_path
                        .parent()
                        .unwrap()
                        .join(v.to_owned().to_lowercase() + ".toml"),
                )?;
                let upper_dev = a?;
                base_spec.arch = upper_dev.arch;
                for base_dev_name in upper_dev.base {
                    if !base_spec.base.contains(&base_dev_name) {
                        base_spec.base.push(base_dev_name.to_owned())
                    }
                }
                base_spec.dev_class = upper_dev.dev_class;
                base_spec.dev_type = upper_dev.dev_type;
                base_spec.name = upper_dev.name;

                // staple on enums, types, messages, settings, and setting commands.
                for enum_ in upper_dev.enums.iter() {
                    base_spec
                        .enums
                        .insert(enum_.0.to_owned(), enum_.1.to_owned());
                }
                for type_ in upper_dev.types.iter() {
                    base_spec
                        .types
                        .insert(type_.0.to_owned(), type_.1.to_owned());
                }
                for msg in upper_dev.msg.iter() {
                    base_spec.msg.insert(msg.0.to_owned(), msg.1.to_owned());
                }
                for stg in upper_dev.settings.iter() {
                    base_spec
                        .settings
                        .insert(stg.0.to_owned(), stg.1.to_owned());
                }
                for stg_cmd in upper_dev.setting_commands.iter() {
                    base_spec
                        .setting_commands
                        .insert(stg_cmd.0.to_owned(), stg_cmd.1.to_owned());
                }

                // update the setting and setting_command enums

                base_spec
                    .enums
                    .insert("SETTING".to_string(), regen_settings_enum(&base_spec));
                base_spec.enums.insert(
                    "SETTING_COMMAND".to_string(),
                    regen_setting_commands_enum(&base_spec),
                );

                Ok(base_spec)
            },
        )
    } else {
        // required to ensure that enum:SETTING and enum:SETTING_COMMAND always exist
        let mut mut_spec = dev_spec.clone();
        mut_spec
            .enums
            .insert("SETTING".to_string(), regen_settings_enum(&mut_spec));
        mut_spec.enums.insert(
            "SETTING_COMMAND".to_string(),
            regen_setting_commands_enum(&mut_spec),
        );
        assign_origins(&mut mut_spec);
        Ok(mut_spec)
    }?;
    Ok(dev)
}
