use std::{fmt::Display, path::Path};

use canandmessage_parser::{utils, DType, Device, Message, Signal, Source};
use clap::{arg, Command};
extern crate canandmessage_parser;

static TEMPLATE: &str = "VERSION \"\"


NS_ :
	NS_DESC_
	CM_
	BA_DEF_
	BA_
	VAL_
	CAT_DEF_
	CAT_
	FILTER
	BA_DEF_DEF_
	EV_DATA_
	ENVVAR_DATA_
	SGTYPE_
	SGTYPE_VAL_
	BA_DEF_SGTYPE_
	BA_SGTYPE_
	SIG_TYPE_REF_
	VAL_TABLE_
	SIG_GROUP_
	SIG_VALTYPE_
	SIGTYPE_VALTYPE_
	BO_TX_BU_
	BA_DEF_REL_
	BA_REL_
	BA_DEF_DEF_REL_
	BU_SG_REL_
	BU_EV_REL_
	BU_BO_REL_
	SG_MUL_VAL_

BS_: 
BU_:";

pub struct DBCBuilder {
    pub dbc: Vec<String>,
    pub dbc_comments: Vec<String>,
    pub float_signals: Vec<String>,
    pub reserved_cnt: u32,
    pub is_public: bool,
}

// having a unified Numer enum lets us preserve precision no matter the input
pub enum Numer {
    Float(f64),
    UInt(u64),
    SInt(i64),
}

impl Display for Numer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Numer::Float(v) => v.fmt(f),
            Numer::UInt(v) => v.fmt(f),
            Numer::SInt(v) => v.fmt(f),
        }
    }
}
impl From<f64> for Numer {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}
impl From<u64> for Numer {
    fn from(value: u64) -> Self {
        Self::UInt(value)
    }
}
impl From<i64> for Numer {
    fn from(value: i64) -> Self {
        Self::SInt(value)
    }
}

// BO_ 1024 NewMessage0: 8 NewNode0
//  SG_ FloatSignal0 : 0|32@1- (1,0) [0|0] "" Vector__XXX
//  SG_ FloatSignal1 : 32|32@1- (1,0) [0|0] "" Vector__XXX
//
//
// SIG_VALTYPE_ 1024 FloatSignal0 : 1;
// SIG_VALTYPE_ 1024 FloatSignal1 : 1;

impl DBCBuilder {
    pub fn new(is_public: bool) -> Self {
        Self {
            dbc: vec![TEMPLATE.to_string()],
            dbc_comments: Vec::new(),
            float_signals: Vec::new(),
            reserved_cnt: 0,
            is_public,
        }
    }
    pub fn add_float_sig(&mut self, full_id: u32, name: &String) {
        self.float_signals
            .push(format!("SIG_VALTYPE_ {full_id} {name} : 1;\n"))
    }

    pub fn render_sg(
        &mut self,
        pos: &mut u32,
        name: &String,
        width: usize,
        signed: bool,
        _scale: Option<f64>,
        _offset: Option<f64>,
        min: Numer,
        max: Numer,
        dest: &String,
        full_id: u32,
        comment: &String,
    ) {
        let sgn = if signed { "-" } else { "+" };
        let scale = _scale.unwrap_or(1.0);
        let offset = _offset.unwrap_or(0.0);
        self.dbc.push(format!(
            " SG_ {name} : {pos}|{width}@1{sgn} ({scale},{offset}) [{min}|{max}] \"\" {dest}\n"
        ));

        let comment = comment.replace("\n", " ");
        self.dbc_comments
            .push(format!("\nCM_ SG_ {full_id} {name} \"{comment}\";"));
        *pos += width as u32;
    }

    pub fn render_signal(
        &mut self,
        pos: &mut u32,
        dev: &Device,
        sig: &Signal,
        sig_prefix: Option<String>,
        dest: &String,
        full_id: u32,
    ) {
        let name = format!(
            "{}{}",
            sig_prefix.as_ref().unwrap_or(&"".to_string()),
            sig.name
        );
        match &sig.dtype {
            DType::None => {
                return;
            }
            DType::UInt { meta } => self.render_sg(
                pos,
                &name,
                meta.width,
                false,
                Some((meta.factor_num as f64) / (meta.factor_den as f64)),
                None,
                meta.min.unwrap_or(0).into(),
                meta.max
                    .unwrap_or(utils::default_uint_max(meta.width))
                    .into(),
                &dest,
                full_id,
                &sig.comment,
            ),
            DType::SInt { meta } => self.render_sg(
                pos,
                &name,
                meta.width,
                true,
                Some((meta.factor_num as f64) / (meta.factor_den as f64)),
                None,
                meta.min
                    .unwrap_or(utils::default_sint_min(meta.width))
                    .into(),
                meta.max
                    .unwrap_or(utils::default_sint_max(meta.width))
                    .into(),
                &dest,
                full_id,
                &sig.comment,
            ),
            DType::Buf { meta } => self.render_sg(
                pos,
                &name,
                meta.width,
                false,
                None,
                None,
                0.0.into(),
                utils::default_uint_max(meta.width).into(),
                &dest,
                full_id,
                &sig.comment,
            ),
            DType::Float { meta } => {
                self.add_float_sig(full_id, &name);
                self.render_sg(
                    pos,
                    &name,
                    meta.width,
                    false,
                    Some((meta.factor_num as f64) / (meta.factor_den as f64)),
                    None,
                    0.0.into(),
                    0.0.into(),
                    &dest,
                    full_id,
                    &sig.comment,
                );
            }
            DType::Bitset { meta } => {
                //self.render_sg(pos, &name, meta.width, false,
                //None, None,
                //0.0.into(), utils::default_uint_max(meta.width).into(),
                //&dest, full_id, &sig.comment);
                let mut max_bit = 0usize;
                for flag in &meta.flags {
                    self.render_sg(
                        pos,
                        &format!("{name}_{}", flag.name),
                        1,
                        false,
                        None,
                        None,
                        0i64.into(),
                        1i64.into(),
                        &dest,
                        full_id,
                        &flag.comment,
                    );
                    max_bit = max_bit.max(flag.bit_idx as usize);
                }

                max_bit += 1;

                if max_bit < meta.width {
                    self.render_sg(
                        pos,
                        &format!("{name}_reserved_bits"),
                        meta.width - max_bit,
                        false,
                        None,
                        None,
                        0.0.into(),
                        utils::default_uint_max(meta.width - max_bit).into(),
                        &dest,
                        full_id,
                        &sig.comment,
                    );
                }
            }
            DType::Pad { width } => self.render_sg(
                pos,
                &name,
                *width,
                false,
                None,
                None,
                0.0.into(),
                utils::default_uint_max(*width).into(),
                &dest,
                full_id,
                &sig.comment,
            ),
            DType::Bool { .. } => {
                self.render_sg(
                    pos,
                    &name,
                    1,
                    false,
                    None,
                    None,
                    0.0.into(),
                    1.0.into(),
                    &dest,
                    full_id,
                    &sig.comment,
                );
            }
            DType::Enum { meta } => self.render_sg(
                pos,
                &name,
                meta.width,
                false,
                None,
                None,
                0.0.into(),
                utils::default_uint_max(meta.width).into(),
                &dest,
                full_id,
                &sig.comment,
            ),
            DType::Struct { meta } => {
                let prefix = match &sig_prefix {
                    Some(p) => format!("{}{}_", p.clone(), meta.name),
                    None => format!("{}_", meta.name),
                };

                meta.signals.iter().for_each(|sig| {
                    self.render_signal(pos, dev, sig, Some(prefix.clone()), dest, full_id)
                });
            }
        };
    }

    pub fn render_message(&mut self, dev_id: u8, dev: &Device, msg: &Message, msg_name: &String) {
        //         return (deviceType << 24) | (REDUX_CAN_ID << 16) | (prodId << 11) | (msgId << 6) | (devId);
        let full_id = (1u32 << 31)
            | ((dev.dev_type as u32) << 24)
            | (0xe << 16)
            | ((dev.dev_class as u32) << 11)
            | ((msg.id as u32) << 6)
            | dev_id as u32;
        let (msg_source, msg_dest) = match msg.source {
            Source::Device => (dev.name.to_lowercase(), "Vector__XXX".to_string()),
            Source::Host => ("Vector__XXX".to_string(), dev.name.to_lowercase()),
            Source::Both => ("Vector__XXX".to_string(), dev.name.to_lowercase()),
        };
        let length = msg.max_length;
        self.dbc.push(format!(
            "\nBO_ {full_id} {name}: {length} {msg_source}\n",
            name = msg_name.to_lowercase()
        ));

        let comment = msg.comment.replace("\n", " ");

        self.dbc_comments.push(format!(
            "\nCM_ BO_ {full_id} {name} \"{comment}\";",
            name = msg_name.to_lowercase(),
            comment = comment
        ));
        let mut pos = 0u32;
        msg.signals.iter().for_each(|sig| {
            self.render_signal(&mut pos, dev, sig, None, &msg_dest, full_id);
        });
    }

    pub fn render_device(&mut self, dev: &Device, dev_id: u8) {
        self.dbc.push(format!(" {}\n", dev.name.to_lowercase()));
        //dev.messages.iter().for_each(|(msg_name, msg)| {
        //    self.render_message(dev_id, dev, msg, msg_name)
        //});
        let mut msg_sorted: Vec<(&String, &Message)> = dev.messages.iter().collect();
        msg_sorted.sort_by_key(|(_, msg)| u8::MAX - msg.id);
        msg_sorted.iter().for_each(|(msg_name, msg)| {
            if !msg.is_public && self.is_public {
                return;
            }
            self.render_message(dev_id, dev, msg, msg_name);
        });

        self.dbc.push("\n".to_string());
        self.dbc.push(self.float_signals.join(""));
        self.dbc.push("\n".to_string());
        self.dbc.push(self.dbc_comments.join(""));
    }
}

impl ToString for DBCBuilder {
    fn to_string(&self) -> String {
        self.dbc.join("")
    }
}

fn main() {
    //let argv: Vec<String> = env::args().collect();

    //let usage = format!("usage: {} toml_folder dbc_folder [device_id]", argv[0]);

    //let folder_name = argv.get(1).expect(usage.as_str());
    //let dbc_folder = argv.get(2).expect(usage.as_str());
    //let dev_id = argv.get(3).unwrap_or(&"0".to_string()).parse::<u8>().expect("device id must be a u8 from [0..=63]");

    let m = Command::new("dbcgen")
        .author("guineawheek guineawheek@gmail.com")
        .version("1.0.0")
        .about("generates dbcs")
        .arg(arg!(--"public" "Filter for public messages only"))
        .arg(arg!(--"dev-id" <ID> "CAN device id to use, defaults to 0"))
        .arg(arg!([toml_folder] "messages folder"))
        .arg(arg!([dbc_folder] "dbc folder"))
        .get_matches();

    let dev_id = m
        .get_one::<String>("dev-id")
        .unwrap_or(&"0".to_string())
        .parse::<u8>()
        .expect("device id must be a u8 from [0..=63]");
    let is_public = m.get_flag("public");
    let folder_name = m.get_one::<String>("toml_folder").unwrap();
    let dbc_folder = m.get_one::<String>("dbc_folder").unwrap();

    for path in std::fs::read_dir(folder_name).unwrap() {
        let path_buf = path.unwrap().path();
        if path_buf
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
            != "toml"
        {
            continue;
        }

        let devspec = canandmessage_parser::parse_spec(&path_buf.as_path()).unwrap();
        let dev: Device = devspec.clone().into();
        let mut dbc = DBCBuilder::new(is_public);
        dbc.render_device(&dev, dev_id);

        std::fs::write(
            Path::new(&format!("{dbc_folder}/{}.dbc", dev.name.to_lowercase())),
            dbc.to_string().as_str(),
        )
        .unwrap();
    }
}
