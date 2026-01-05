use crate::toml_defs::{self, DeviceSpec, TypeSpec};
use crate::utils::{
    decode_bounds_f64, default_sint_max, default_sint_min, default_uint_max, opt_value_to_opt_bool,
    opt_value_to_opt_f64, opt_value_to_opt_i64, opt_value_to_opt_u64, read_suffix,
    read_suffix_as_usize,
};
use crate::{BitsetMeta, DType, Device, EnumMeta, Message, Setting, Signal, Source, StructMeta};

//pub mod model;

impl DType {
    pub fn bit_length(&self) -> usize {
        match self {
            DType::None => 0,
            DType::UInt { meta } => meta.width,
            DType::SInt { meta } => meta.width,
            DType::Buf { meta } => meta.width,
            DType::Float { meta } => meta.width,
            DType::Bitset { meta } => meta.width,
            DType::Pad { width } => *width,
            DType::Bool { default_value: _ } => 1,
            DType::Enum { meta } => meta.width,
            DType::Struct { meta } => meta
                .signals
                .to_owned()
                .into_iter()
                .map(|x| x.dtype.bit_length())
                .sum(),
        }
    }

    pub fn is_pad(&self) -> bool {
        match self {
            DType::None => true,
            DType::Pad { width: _ } => true,
            _ => false,
        }
    }

    pub fn canonical_name(&self) -> String {
        match self {
            DType::None => panic!("invalid"),
            DType::UInt { meta } => format!("uint:{}", meta.width),
            DType::SInt { meta } => format!("sint:{}", meta.width),
            DType::Buf { meta } => format!("buf:{}", meta.width),
            DType::Float { meta } => format!("float:{}", meta.width),
            DType::Bitset { meta } => format!("uint:{}", meta.width),
            DType::Pad { width } => format!("pad:{}", width),
            DType::Bool { .. } => format!("bool"),
            DType::Enum { meta } => format!("enum:{}", meta.name),
            DType::Struct { meta } => format!("struct:{}", meta.name),
        }
    }

    fn from_type(
        type_name: String,
        type_def: &toml_defs::TypeSpec,
        default_value: &Option<toml::Value>,
        dev: &toml_defs::DeviceSpec,
    ) -> Self {
        let default_value = if default_value.is_none() {
            &type_def.default_value
        } else {
            default_value
        };
        let width = type_def.bits as usize;
        match type_def.btype.as_str() {
            "uint" => {
                let min = opt_value_to_opt_u64(&type_def.min).and_then(|v| {
                    if v == 0 {
                        None
                    } else {
                        Some(v)
                    }
                });
                let max = opt_value_to_opt_u64(&type_def.max).and_then(|v| {
                    if v == default_uint_max(width) {
                        None
                    } else {
                        Some(v)
                    }
                });
                DType::UInt {
                    meta: crate::UIntMeta {
                        width,
                        min,
                        max,

                        default_value: opt_value_to_opt_u64(default_value).unwrap_or(0),
                        factor_num: type_def.factor[0],
                        factor_den: type_def.factor[1],
                    },
                }
            }
            "sint" => {
                let min = opt_value_to_opt_i64(&type_def.min).and_then(|v| {
                    if v == default_sint_min(width) {
                        None
                    } else {
                        Some(v)
                    }
                });
                let max = opt_value_to_opt_i64(&type_def.max).and_then(|v| {
                    if v == default_sint_max(width) {
                        None
                    } else {
                        Some(v)
                    }
                });
                DType::SInt {
                    meta: crate::SIntMeta {
                        width,
                        min,
                        max,

                        default_value: opt_value_to_opt_i64(default_value).unwrap_or(0i64),
                        factor_num: type_def.factor[0],
                        factor_den: type_def.factor[1],
                    },
                }
            }
            "buf" => DType::Buf {
                meta: crate::BufMeta {
                    width: type_def.bits as usize,
                    default_value: opt_value_to_opt_u64(default_value)
                        .unwrap_or(0)
                        .to_le_bytes(),
                },
            },
            "float" => {
                let (min, max) = decode_bounds_f64(&type_def.min, &type_def.max);
                DType::Float {
                    meta: crate::FloatMeta {
                        width,
                        min,
                        max,
                        default_value: opt_value_to_opt_f64(default_value).unwrap_or(0f64),
                        allow_nan_inf: type_def.allow_nan_inf,
                        factor_num: type_def.factor[0],
                        factor_den: type_def.factor[1],
                    },
                }
            }
            "bitset" => DType::Bitset {
                meta: crate::BitsetMeta::from(&type_name, &type_def),
            },
            "pad" => DType::Pad {
                width: type_def.bits as usize,
            },
            "bool" => DType::Bool {
                default_value: opt_value_to_opt_u64(default_value).unwrap_or(0) > 0,
            },
            "struct" => DType::Struct {
                meta: StructMeta::from(&type_name, &type_def, dev),
            },
            _ => DType::from_type(
                type_def.btype.to_owned(),
                dev.types
                    .get(&type_def.btype)
                    .expect("undefined btype defined in types"),
                default_value,
                dev,
            ),
        }
    }

    fn from_sig(
        dev: &toml_defs::DeviceSpec,
        dtype_name: &String,
        default_value: &Option<toml::Value>,
    ) -> Self {
        // this function allows "inline" typedefs, unlike from_type
        if dtype_name == "none" {
            DType::None
        } else if dtype_name.starts_with("buf:") {
            DType::Buf {
                meta: crate::BufMeta {
                    width: read_suffix_as_usize(dtype_name),
                    default_value: opt_value_to_opt_u64(default_value)
                        .unwrap_or(0u64)
                        .to_le_bytes(),
                },
            }
        } else if dtype_name.starts_with("uint:") {
            let width = read_suffix_as_usize(dtype_name);
            DType::UInt {
                meta: crate::UIntMeta {
                    width: width,
                    min: None,
                    max: None,
                    default_value: opt_value_to_opt_u64(default_value).unwrap_or(0u64),
                    factor_num: 1,
                    factor_den: 1,
                },
            }
        } else if dtype_name.starts_with("sint") {
            let width = read_suffix_as_usize(dtype_name);
            DType::SInt {
                meta: crate::SIntMeta {
                    width: width,
                    min: None,
                    max: None,
                    default_value: opt_value_to_opt_i64(default_value).unwrap_or(0i64),
                    factor_num: 1,
                    factor_den: 1,
                },
            }
        } else if dtype_name.starts_with("float:") {
            DType::Float {
                meta: crate::FloatMeta {
                    width: read_suffix_as_usize(dtype_name),
                    min: None,
                    max: None,
                    default_value: opt_value_to_opt_f64(default_value).unwrap_or(0f64),
                    allow_nan_inf: true,
                    factor_num: 1,
                    factor_den: 1,
                },
            }
        } else if dtype_name.starts_with("pad:") {
            DType::Pad {
                width: read_suffix_as_usize(dtype_name),
            }
        } else if dtype_name == "bool" {
            DType::Bool {
                default_value: opt_value_to_opt_bool(default_value).unwrap_or(false),
            }
        } else if dtype_name == "setting_data" {
            DType::Buf {
                meta: crate::BufMeta {
                    width: 48,
                    default_value: [0u8; 8],
                },
            }
        } else if dtype_name.starts_with("enum:") {
            let name = read_suffix(dtype_name);
            let ent = dev
                .enums
                .get(&name)
                .expect(&format!("undefined enum {}", dtype_name));
            DType::Enum {
                meta: EnumMeta::from(
                    &name,
                    ent,
                    default_value.as_ref().map_or(None, |v| match v {
                        toml::Value::String(s) => Some(s.to_owned()),
                        _ => None,
                    }),
                ),
            }
        } else {
            // this branch handles struct/alias types
            // but we need to overwrite it with our own default value
            DType::from_type(
                dtype_name.to_owned(),
                dev.types.get(dtype_name).expect(&format!(
                    "undefined type {} referenced in signal",
                    dtype_name
                )),
                &default_value,
                dev,
            )
        }
    }
}

// TODO: add mux support. i can't be assed to do this
impl Signal {
    fn from(sgnl: &toml_defs::MessageSignalSpec, dev: &toml_defs::DeviceSpec) -> Self {
        Self {
            name: sgnl.name.to_owned(),
            comment: sgnl.comment.to_owned(),
            dtype: DType::from_sig(dev, &sgnl.dtype, &sgnl.default_value),
            optional: sgnl.optional,
        }
    }
    pub fn from_stg(name: &String, stg: &Setting) -> Self {
        Self {
            name: name.to_owned(),
            comment: stg.comment.to_owned(),
            dtype: stg.dtype.clone(),
            optional: false,
        }
    }
}
impl From<&Setting> for Signal {
    fn from(value: &Setting) -> Self {
        Signal {
            name: "value".to_string(),
            comment: "setting value".to_string(),
            dtype: value.dtype.clone(),
            optional: false,
        }
    }
}

impl Source {
    pub fn flip(&self) -> Source {
        match &self {
            Source::Device => Source::Host,
            Source::Host => Source::Device,
            Source::Both => Source::Both,
        }
    }
}

impl From<&String> for Source {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "device" => Source::Device,
            "host" => Source::Host,
            "bidir" => Source::Both,
            "both" => Source::Both,
            &_ => todo!(),
        }
    }
}

impl Message {
    fn from(dm: &toml_defs::DeviceMessageSpec, dev: &toml_defs::DeviceSpec) -> Self {
        let (min_length, max_length) = match dm.length {
            Some(len) => (len, len),
            None => (dm.min_length.unwrap_or(0u8), dm.max_length.unwrap_or(8u8)),
        };

        Message {
            id: dm.id,
            min_length: min_length,
            max_length: max_length,
            comment: dm.comment.to_owned(),
            is_public: dm.is_public,
            signals: dm.signals.iter().map(|v| Signal::from(v, dev)).collect(),
            source: (&dm.source).into(),
            origin_lname: dev.name.to_lowercase(),
        }
    }
}

impl Setting {
    fn from(
        name: String,
        value: &toml_defs::DeviceSettingSpec,
        dev: &toml_defs::DeviceSpec,
    ) -> Self {
        let dtype = DType::from_sig(dev, &value.dtype, &value.default_value);
        Setting {
            name: name.to_owned(),
            id: value.id,
            comment: value.comment.to_owned(),
            // god this is a hack
            dtype,
            readable: value.readable,
            writable: value.writable,
            reset_on_default: value.reset_on_default,
            special_flags: value.special_flags.clone(),
            origin_lname: dev.name.to_lowercase(),
            vendordep: value.vendordep,
            vdep_setting: value.vdep_setting,
        }
    }
}

impl EnumMeta {
    fn from(name: &String, entry: &toml_defs::EnumSpec, default_value: Option<String>) -> Self {
        let default_value = default_value.unwrap_or(entry.default_value.clone());
        EnumMeta {
            name: name.to_owned(),
            origin_lname: entry.origin_lname.to_owned(),
            width: entry.bits as usize,
            default_value: match entry.values.get(&default_value) {
                Some(_) => entry.default_value.clone(),
                _ => match name.as_str() {
                    "SETTING" | "SETTING_COMMAND" => "".to_string(),
                    _ => panic!(
                        "invalid enum default value {}::{}",
                        name, entry.default_value
                    ),
                },
            },
            is_public: entry.is_public,
            values: entry
                .values
                .iter()
                .map(|(ent_name, ent)| {
                    (
                        ent.id as u64,
                        crate::EnumEntry {
                            name: ent_name.to_owned(),
                            comment: ent.comment.to_owned(),
                            index: ent.id as u64,
                        },
                    )
                })
                .collect(),
        }
    }
}

impl BitsetMeta {
    pub fn default_u64(&self) -> u64 {
        let mut v = 0u64;

        let _: Vec<()> = self
            .flags
            .iter()
            .map(|ent| {
                v |= (ent.default_value as u64) << ent.bit_idx;
            })
            .collect();

        v
    }

    pub fn from(name: &String, type_def: &TypeSpec) -> Self {
        let default_value = opt_value_to_opt_u64(&type_def.default_value).unwrap_or(0u64);
        crate::BitsetMeta {
            name: name.to_owned(),
            origin_lname: type_def.origin_lname.to_owned(),
            width: type_def.bits as usize,
            flags: type_def
                .bit_flags
                .iter()
                .enumerate()
                .map(|(i, x)| crate::BitsetFlag {
                    bit_idx: i as u32,
                    default_value: ((default_value >> i) & 1) > 0,
                    name: x.name.to_owned(),
                    comment: x.comment.to_owned(),
                })
                .collect(),
        }
    }
}

impl StructMeta {
    pub fn from(name: &String, ent: &TypeSpec, dev: &DeviceSpec) -> Self {
        StructMeta {
            name: name.to_owned(),
            origin_lname: ent.origin_lname.to_owned(),
            signals: ent
                .signals
                .iter()
                .map(|sig| Signal {
                    name: sig.name.to_owned(),
                    comment: sig.comment.to_owned(),
                    dtype: DType::from_sig(dev, &sig.dtype, &sig.default_value),
                    optional: sig.optional,
                })
                .collect(),
        }
    }
}

impl From<toml_defs::DeviceSpec> for Device {
    fn from(dev_spec: toml_defs::DeviceSpec) -> Self {
        let dev_spec_local = dev_spec.clone();
        Device {
            name: dev_spec.name.to_owned(),
            arch: dev_spec.arch.to_owned(),
            dev_type: dev_spec.dev_type,
            dev_class: dev_spec.dev_class,
            java_package: dev_spec
                .vendordep
                .as_ref()
                .map_or("".to_owned(), |v| v.java_package.to_owned()),
            cpp_namespace: dev_spec
                .vendordep
                .as_ref()
                .map_or("".to_owned(), |v| v.cpp_namespace.to_owned()),

            messages: dev_spec_local
                .msg
                .iter()
                .map(|msg| (msg.0.to_owned(), Message::from(msg.1, &dev_spec_local)))
                .collect(),
            settings: dev_spec_local
                .settings
                .iter()
                .map(|stg| {
                    (
                        stg.0.to_owned(),
                        Setting::from(stg.0.to_owned(), stg.1, &dev_spec_local),
                    )
                })
                .collect(),
            enums: dev_spec_local
                .enums
                .iter()
                .map(|(name, ent)| (name.to_owned(), EnumMeta::from(name, ent, None)))
                .collect(),
            bitsets: dev_spec_local
                .types
                .iter()
                .filter_map(|(name, ent)| match ent.btype.as_str() {
                    "bitset" => Some((name.to_owned(), crate::BitsetMeta::from(name, ent))),
                    _ => None,
                })
                .collect(),
            structs: dev_spec_local
                .types
                .iter()
                .filter_map(|(name, ent)| match ent.btype.as_str() {
                    "struct" => Some((
                        name.to_owned(),
                        StructMeta::from(name, ent, &dev_spec_local),
                    )),
                    _ => None,
                })
                .collect(),
        }
    }
}
