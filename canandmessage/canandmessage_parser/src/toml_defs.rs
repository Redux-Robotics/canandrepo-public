use serde::Deserialize;
use std::collections::BTreeMap;
use toml::Value;

fn default_true() -> bool {
    true
}
fn default_uint() -> String {
    String::from("uint")
}

fn default_scale() -> [i64; 2] {
    [1, 1]
}
#[derive(Deserialize, Debug, Clone)]
pub struct DeviceSpec {
    pub name: String,
    pub base: Vec<String>,
    pub arch: String,
    #[serde(default = "default_true")]
    pub is_public: bool,
    pub dev_type: u8,
    pub dev_class: u8,
    pub msg: BTreeMap<String, DeviceMessageSpec>,
    #[serde(default = "BTreeMap::new")]
    pub settings: BTreeMap<String, DeviceSettingSpec>,
    #[serde(default = "BTreeMap::new")]
    pub types: BTreeMap<String, TypeSpec>,
    #[serde(default = "BTreeMap::new")]
    pub enums: BTreeMap<String, EnumSpec>,
    #[serde(default = "BTreeMap::new")]
    pub setting_commands: BTreeMap<String, SettingCommandSpec>,

    pub vendordep: Option<VendordepSpec>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct VendordepSpec {
    pub java_package: String,
    pub cpp_namespace: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeviceMessageSpec {
    pub id: u8,
    pub min_length: Option<u8>,
    pub max_length: Option<u8>,
    pub length: Option<u8>,
    pub source: String,
    #[serde(default = "default_true")]
    pub is_public: bool,
    #[serde(default = "default_true")]
    pub vendordep: bool,
    pub comment: String,
    pub signals: Vec<MessageSignalSpec>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct MessageSignalSpec {
    pub name: String,
    pub comment: String,
    pub dtype: String,
    #[serde(default = "bool::default")]
    pub optional: bool,
    pub default_value: Option<Value>,

    #[serde(default = "bool::default")]
    pub mux: bool,
    pub muxed_by: Option<String>,
    pub muxed_match: Option<Value>, // TODO: this isn't correct

    #[serde(default = "default_true")]
    pub alchemist: bool,
}

#[derive(Deserialize, Debug, Clone)]
pub struct DeviceSettingSpec {
    pub id: u8,
    pub comment: String,
    pub dtype: String,
    pub default_value: Option<Value>,

    #[serde(default = "default_true")]
    pub is_public: bool,
    #[serde(default = "default_true")]
    pub vendordep: bool,
    #[serde(default = "default_true")]
    pub vdep_setting: bool,
    #[serde(default = "default_true")]
    pub readable: bool,
    #[serde(default = "default_true")]
    pub writable: bool,
    #[serde(default = "default_true")]
    pub reset_on_default: bool,
    #[serde(default = "Vec::default")]
    pub special_flags: Vec<String>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct TypeSpec {
    pub btype: String,
    #[serde(default = "String::default")]
    pub utype: String,

    #[serde(skip, default = "String::default")]
    pub origin_lname: String,

    pub bits: u8,
    pub min: Option<Value>,
    pub max: Option<Value>,
    #[serde(default = "default_true")]
    pub allow_nan_inf: bool,
    pub default_value: Option<Value>,
    #[serde(default = "default_scale")]
    pub factor: [i64; 2],
    pub offset: Option<Value>,
    #[serde(default = "Vec::default")]
    pub signals: Vec<MessageSignalSpec>,
    #[serde(default = "Vec::default")]
    pub bit_flags: Vec<BitsetFlagSpec>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct SettingCommandSpec {
    pub id: u8,
    #[serde(default = "default_true")]
    pub vendordep: bool,
    pub comment: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct BitsetFlagSpec {
    pub name: String,
    pub comment: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnumSpec {
    #[serde(default = "default_uint")]
    pub btype: String,
    pub bits: u8,
    #[serde(default = "default_true")]
    pub is_public: bool,
    #[serde(skip, default = "String::default")]
    pub origin_lname: String,
    pub default_value: String,
    pub values: BTreeMap<String, EnumEntrySpec>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct EnumEntrySpec {
    pub id: u32,
    pub comment: String,
}
