use std::time::{Duration, Instant};

use canandmessage::{cananddevice, traits::CanandDeviceSetting};
use fifocore::ReduxFIFOMessage;
use frc_can_id::{FRCCanDeviceType, FRCCanId};
use rustc_hash::FxHashMap;
use serial_numer::{ProductId, SerialNumer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConflictPacket {
    serial: SerialNumer,
    timestamp: Instant,
}

impl ConflictPacket {
    /// conflict packets can be up to 2.5 seconds old
    pub fn current(&self, ts: Instant) -> bool {
        self.timestamp + Duration::from_millis(2500) > ts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum ReduxDeviceType {
    Encoder,
    Gyroscope,
    MotorController,
    ColorDistanceSensor,
    Other(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct DeviceKey {
    pub dev_type: ReduxDeviceType,
    pub dev_id: u8,
}

impl From<FRCCanId> for DeviceKey {
    fn from(value: FRCCanId) -> Self {
        let dev_type = FRCCanDeviceType::from(value.device_type_code());

        let device_type = match dev_type {
            FRCCanDeviceType::MotorController => ReduxDeviceType::MotorController,
            FRCCanDeviceType::GyroSensor => ReduxDeviceType::Gyroscope,
            FRCCanDeviceType::DistanceSensor => ReduxDeviceType::ColorDistanceSensor,
            FRCCanDeviceType::Encoder => ReduxDeviceType::Encoder,
            other => ReduxDeviceType::Other(other.as_u8()),
        };

        Self {
            dev_type: device_type,
            dev_id: value.device_number(),
        }
    }
}

impl DeviceKey {
    pub fn pretty_str(&self) -> String {
        format!("{:?}:{}", self.dev_type, self.dev_id)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct DeviceName {
    name0: Option<[u8; 6]>,
    name1: Option<[u8; 6]>,
    name2: Option<[u8; 6]>,
}

impl DeviceName {
    pub fn name(&self) -> Option<String> {
        let name0 = self.name0?;
        let name1 = self.name1?;
        let name2 = self.name2?;

        let s: Vec<u8> = name0
            .iter()
            .chain(name1.iter())
            .chain(name2.iter())
            .take_while(|v| **v != 0)
            .copied()
            .collect();
        Some(String::from_utf8_lossy(&s).into_owned())
    }
}

/// collection of information about a specific can id
#[derive(Debug, PartialEq, Clone)]
pub struct Device {
    id: DeviceKey,
    // used to determine exact product sku
    serial_numer: Option<SerialNumer>,
    // most recent active packet
    // used to determine presence
    most_recent_active: Option<Instant>,

    firmware_version: Option<cananddevice::types::FirmwareVersion>,
    device_type: Option<u16>,
    bootloader: bool,
    setting_cache: FxHashMap<u8, [u8; 6]>,

    conflict_packets: Vec<ConflictPacket>,
    authorized_serial: Option<SerialNumer>,
}

impl Device {
    pub fn new(id: DeviceKey) -> Self {
        Self {
            id,
            serial_numer: None,
            most_recent_active: None,
            firmware_version: None,
            device_type: None,
            bootloader: false,
            setting_cache: FxHashMap::default(),
            conflict_packets: Vec::new(),
            authorized_serial: None,
        }
    }

    pub fn setting_cache(&self) -> &FxHashMap<u8, [u8; 6]> {
        &self.setting_cache
    }

    pub fn setting_cache_mut(&mut self) -> &mut FxHashMap<u8, [u8; 6]> {
        &mut self.setting_cache
    }

    fn update_recent_active(&mut self, ts: Instant) {
        self.most_recent_active = Some(self.most_recent_active.map_or(ts, |v| ts.max(v)));
    }

    pub fn set_arb_serial(&mut self, serial: SerialNumer) {
        if !(serial.is_zero() || serial.is_unset()) {
            self.authorized_serial = Some(serial);
        } else {
            self.authorized_serial = None;
        }
    }

    /// called whenever an id is set on an authorized device
    pub fn set_arb_serial_as_diff_id(&mut self) {
        if let Some(serial) = self.authorized_serial {
            self.conflict_packets.retain(|s| s.serial != serial);
        }
        self.authorized_serial = None;
    }

    pub fn handle_msg(&mut self, msg: &ReduxFIFOMessage) {
        let frame = canandmessage::CanandMessageWrapper(msg.clone());
        let now = Instant::now();
        let mut is_conflict_packet = false;
        if let Ok(device_msg) = TryInto::<cananddevice::Message>::try_into(frame) {
            match device_msg {
                cananddevice::Message::CanIdError { addr_value } => {
                    is_conflict_packet = true;
                    let serial = addr_value.into();
                    if self
                        .conflict_packets
                        .iter_mut()
                        .find_map(|p| {
                            if p.serial == serial {
                                p.timestamp = p.timestamp.max(now);
                                Some(())
                            } else {
                                None
                            }
                        })
                        .is_none()
                    {
                        self.conflict_packets.push(ConflictPacket {
                            serial,
                            timestamp: now,
                        });
                    }
                }
                cananddevice::Message::Enumerate {
                    serial,
                    is_bootloader,
                } => {
                    self.serial_numer = Some(SerialNumer::new(serial));
                    self.bootloader = is_bootloader;
                }
                cananddevice::Message::ReportSetting {
                    address,
                    value,
                    ..
                } => {
                    self.setting_cache.insert(address as u8, value);
                    match address {
                        cananddevice::types::Setting::SerialNumber => {
                            self.serial_numer = Some(SerialNumer::new(value));
                        }
                        cananddevice::types::Setting::FirmwareVersion => {
                            if let Some(cananddevice::Setting::FirmwareVersion(version)) =
                                cananddevice::Setting::from_address_data(address, &value).ok()
                            {
                                self.firmware_version = Some(version);
                            }
                        }
                        cananddevice::types::Setting::DeviceType => {
                            if let Some(cananddevice::Setting::DeviceType(dtype)) =
                                cananddevice::Setting::from_address_data(address, &value).ok()
                            {
                                self.device_type = Some(dtype);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        } else {
            let id = FRCCanId(msg.message_id);
            if id.api_index() == cananddevice::MessageIndex::ReportSetting as u16 {
                self.setting_cache
                    .insert(msg.data[0], msg.data[1..7].try_into().unwrap());
            }
        }
        if !is_conflict_packet {
            self.update_recent_active(now);
        }
    }

    pub fn poll(&mut self, ts: Instant) {
        self.conflict_packets.retain(|ent| ent.current(ts));
    }

    pub fn still_on_bus(&mut self, ts: Instant) -> bool {
        !self.conflict_packets.is_empty()
            || self
                .most_recent_active
                .map_or(false, |t| (ts - t) <= Duration::from_secs(2))
    }

    pub fn dev_type(&self, ts: Instant) -> DeviceType {
        // if we're in conflict, we're in conflict.
        if self.conflict_packets.iter().any(|ent| ent.current(ts)) {
            return DeviceType::InConflict(InConflict {
                dev_id: self.id,
                devices_detected: self.conflict_packets.iter().map(|ent| ent.serial).collect(),
                authorized: self.authorized_serial,
            });
        }

        let Some(serial) = self.serial_numer else {
            // our ability to guess is greatly limited now
            if self.id.dev_type == ReduxDeviceType::Encoder {
                // really old canandmags don't support enumerate
                return DeviceType::Canandmag(DeviceVariant::Legacy);
            } else {
                return DeviceType::NotSure(self.id.dev_type);
            }
        };

        if self.bootloader {
            return match serial.product_id() {
                ProductId::Encoder => DeviceType::Canandmag(DeviceVariant::Bootloader),
                ProductId::Gyro => DeviceType::Canandgyro(DeviceVariant::Bootloader),
                ProductId::Sandworm => DeviceType::Canandcolor(DeviceVariant::Bootloader),
                ProductId::Nitrate => DeviceType::Nitrate(DeviceVariant::Bootloader),
                _ => DeviceType::Unknown(serial),
            };
        }

        match serial.product_id() {
            ProductId::Encoder => DeviceType::Canandmag(DeviceVariant::Legacy),
            ProductId::Gyro => DeviceType::Canandgyro(DeviceVariant::Legacy),
            ProductId::Sandworm => DeviceType::Canandcolor(DeviceVariant::Legacy),
            ProductId::Nitrate => DeviceType::Nitrate(DeviceVariant::Fd),
            _ => DeviceType::Unknown(serial),
        }
    }

    pub fn in_conflict(&self) -> bool {
        !self.conflict_packets.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceVariant {
    // No CAN-FD
    Legacy,
    // No main functionality
    Bootloader,
    // Fd-capable variant
    Fd,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DeviceType {
    /// we have no idea what it is, but we know it's in can id conflict
    InConflict(InConflict),
    /// we don't have a serial numer yet
    NotSure(ReduxDeviceType),
    Canandmag(DeviceVariant),
    Canandcolor(DeviceVariant),
    Canandgyro(DeviceVariant),
    Nitrate(DeviceVariant),
    Unknown(SerialNumer),
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct InConflict {
    dev_id: DeviceKey,
    devices_detected: Vec<SerialNumer>,
    /// Serial numer of the authorized device, if any
    /// This is only set by the rest API.
    authorized: Option<SerialNumer>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_device_type() {
        let s = serde_json::to_string(&DeviceType::InConflict(InConflict {
            dev_id: DeviceKey {
                dev_type: ReduxDeviceType::ColorDistanceSensor,
                dev_id: 15,
            },
            devices_detected: vec![SerialNumer::new([1, 2, 3, 4, 5, 6])],
            authorized: None,
        }))
        .unwrap();
        panic!("{s}");
    }
}
