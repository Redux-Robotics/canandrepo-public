import typing
from .serde import Serde, Anything

def default_true() -> bool:
    return True

def default_false() -> bool:
    return False

def default_uint() -> str:
    return "uint"

def default_scale() -> list :
    return [1,1]

#[derive(Deserialize, Debug, Clone)]
class DeviceSpec(Serde):
    name: str
    base: typing.List[str]
    arch: str
    #[serde(default = "default_true")]
    is_public: bool = default_true
    dev_type: int
    dev_class: int
    msg: typing.Dict[str, 'DeviceMessageSpec'] = dict
    settings: typing.Dict[str, 'DeviceSettingSpec'] = dict
    #[serde(default = "BTreeMap::new")]
    types: typing.Dict[str, 'TypeSpec'] = dict
    #[serde(default = "BTreeMap::new")]
    enums: typing.Dict[str, 'EnumSpec'] = dict
    #[serde(default = "BTreeMap::new")]
    setting_commands: typing.Dict[str, 'SettingCommandSpec'] = dict

    vendordep: typing.Optional['VendordepSpec']

#[derive(Deserialize, Debug, Clone)]
class VendordepSpec(Serde):
    java_package: str
    cpp_namespace: str

#[derive(Deserialize, Debug, Clone)]
class DeviceMessageSpec(Serde):
    id: int
    min_length: typing.Optional[int]
    max_length: typing.Optional[int]
    length: typing.Optional[int]
    frame_period_setting: typing.Optional[str]
    source: str
    #[serde(default = "default_true")]
    is_public: bool = default_true
    #[serde(default = "default_true")]
    vendordep: bool = default_true
    comment: str
    signals: typing.List['MessageSignalSpec']

#[derive(Deserialize, Debug, Clone)]
class MessageSignalSpec(Serde):
    name: str
    comment: str
    dtype: str
    #[serde(default = "bool::default")]
    optional: bool = default_false
    default_value: Anything

    #[serde(default = "bool::default")]
    mux: bool = default_false
    muxed_by: typing.Optional[str]
    muxed_match: Anything

#[derive(Deserialize, Debug, Clone)]
class DeviceSettingSpec(Serde):
    id: int
    comment: str
    dtype: str
    default_value: Anything


    #[serde(default = "default_true")]
    is_public: bool = default_true
    #[serde(default = "default_true")]
    vendordep: bool = default_true
    #[serde(default = "default_true")]
    vdep_setting: bool = default_true
    #[serde(default = "default_true")]
    readable: bool = default_true
    #[serde(default = "default_true")]
    writable: bool = default_true
    #[serde(default = "default_true")]
    reset_on_default: bool = default_true
    #[serde(default = "Vec::default")]
    special_flags: typing.List[str] = list


#[derive(Deserialize, Debug, Clone)]
class TypeSpec(Serde):
    btype: str
    comment: str = str
    unit: str = str

    #[serde(default = "String::default")]
    utype: str = str
    bits: int
    min: Anything
    max: Anything
    #[serde(default = "default_true")]
    allow_nan_inf: bool = default_true
    default_value: Anything
    #[serde(default = "default_scale")]
    factor: list = default_scale
    offset: Anything
    #[serde(default = "Vec::default")]
    signals: typing.List[MessageSignalSpec] = list
    #[serde(default = "Vec::default")]
    bit_flags: typing.List['BitsetFlagSpec'] = list

#[derive(Deserialize, Debug, Clone)]
class SettingCommandSpec(Serde):
    id: int
    #[serde(default = "default_true")]
    vendordep: bool = default_true
    comment: str

#[derive(Deserialize, Debug, Clone)]
class BitsetFlagSpec(Serde):
    name: str
    comment: str

#[derive(Deserialize, Debug, Clone)]
class EnumSpec(Serde):
    #[serde(default = "default_uint")]
    comment: str = str
    btype: str = default_uint
    bits: int
    #[serde(default = "default_true")]
    is_public: bool = default_true
    default_value: str
    values: typing.Dict[str, 'EnumEntrySpec']

#[derive(Deserialize, Debug, Clone)]
class EnumEntrySpec(Serde):
    id: int
    comment: str
