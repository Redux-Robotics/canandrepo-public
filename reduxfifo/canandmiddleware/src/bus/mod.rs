use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use canandmessage::traits::CanandDeviceMessage;
use fifocore::{FIFOCore, ReduxFIFOMessage, Session};
use frc_can_id::{FRCCanId, FRCCanVendor, build_frc_can_id};
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use serial_numer::SerialNumer;
use tokio::task::JoinHandle;

use crate::{
    bus::device::{Device, DeviceKey, DeviceType},
    log::log_error,
};

pub mod device;

const fn sanitize_id(id: u32) -> u32 {
    (id & build_frc_can_id(0x1f, 0x00, 0x0, 0x3f)) | 0x0e0000
}

const fn expand<T: Copy, const N: usize, const M: usize>(v: [T; N], p: T) -> [T; M] {
    assert!(M > N);
    let mut dest = [p; M];
    dest.split_at_mut(N).0.copy_from_slice(&v);
    dest
}

#[derive(Debug)]
pub struct BusState {
    /// known devices
    pub devices: FxHashMap<DeviceKey, Device>,
    pub task: JoinHandle<()>,
    pub fifocore: FIFOCore,
    pub bus_id: u16,

    pub stale_device: Option<DeviceKey>,
    pub enumerate_limiter: u32,
}

impl BusState {
    pub fn new(task: JoinHandle<()>, fifocore: FIFOCore, bus_id: u16) -> Self {
        Self {
            devices: Default::default(),
            task,
            fifocore,
            bus_id,
            enumerate_limiter: 0,
            stale_device: None,
        }
    }

    pub fn ingest_buffer(&mut self, msgs: &fifocore::ReadBuffer) {
        for msg in msgs.iter() {
            let can_id = FRCCanId::new(msg.id());
            if can_id.manufacturer() != FRCCanVendor::Redux {
                return;
            }

            let device_key: DeviceKey = can_id.into();
            if let Some(stale) = self.stale_device && stale == device_key {
                // REST has signaled that this device could be a ghost device (e.g. from can id change), so we'll ignore it this loop
                continue;
            }

            if !self.devices.contains_key(&device_key) {
                self.devices.insert(device_key, Device::new(device_key));
            }
            let Some(dev) = self.devices.get_mut(&device_key) else {
                return;
            };
            dev.handle_msg(msg);
        }
        self.stale_device = None;
    }

    pub fn poll(&mut self) {
        let now = Instant::now();
        self.devices.values_mut().for_each(|d| d.poll(now));
        self.devices.retain(|_, d| d.still_on_bus(now));
        if self.enumerate_limiter % 100 == 0 {
            // every half second or so we enumerate the bus.
            let _ = self.enumerate();
        }

        self.enumerate_limiter = self.enumerate_limiter.wrapping_add(1);
    }

    pub fn clear_known_devices(&mut self) {
        self.devices.clear();
    }

    pub fn known_devices(&self) -> FxHashMap<String, DeviceType> {
        let now = Instant::now();
        FxHashMap::from_iter(
            self.devices
                .iter()
                .map(|(k, v)| (k.pretty_str(), v.dev_type(now))),
        )
    }

    pub fn arbitrate(
        &mut self,
        id: u32,
        serial: SerialNumer,
    ) -> Result<(), fifocore::error::Error> {
        let id = sanitize_id(id);

        let mut msg: canandmessage::CanandMessageWrapper<ReduxFIFOMessage> =
            canandmessage::cananddevice::Message::CanIdArbitrate {
                addr_value: serial.into_msg_padded(),
            }
            .try_into_wrapper(id)
            .map_err(|e| {
                log_error!("Could not serialize arbitration message: {e}");
                fifocore::error::Error::BusWriteFail
            })?;
        msg.0.bus_id = self.bus_id;

        self.fifocore.write_single(&msg)?;
        self.enumerate()?;
        // If we know the device exists, we set the known serial number of the device to the one we arbitrate with.
        let key = DeviceKey::from(FRCCanId(id));
        if let Some(entry) = self.devices.get_mut(&key) {
            entry.set_arb_serial(serial);
        }

        Ok(())
    }

    pub fn enumerate(&self) -> Result<(), fifocore::error::Error> {
        let msg = ReduxFIFOMessage::id_data(
            self.bus_id,
            frc_can_id::REDUX_BROADCAST_ENUMERATE,
            [0u8; _],
            0,
            0,
        );
        self.fifocore.write_single(&msg)
    }

    pub fn blink(&self, id: u32, value: u8) -> Result<(), fifocore::error::Error> {
        let id = sanitize_id(id);
        let mut msg: canandmessage::CanandMessageWrapper<ReduxFIFOMessage> =
            canandmessage::cananddevice::Message::PartyMode { party_level: value }
                .try_into_wrapper(id)
                .map_err(|e| {
                    log_error!("Could not serialize blink message: {e}");
                    fifocore::error::Error::BusWriteFail
                })?;
        msg.0.bus_id = self.bus_id;
        self.fifocore.write_single(&msg)?;
        Ok(())
    }

    pub fn set_id(&mut self, id: u32, value: u8) -> Result<(), fifocore::error::Error> {
        let id = sanitize_id(id);
        let mut msg: canandmessage::CanandMessageWrapper<ReduxFIFOMessage> =
            canandmessage::cananddevice::Message::SetSetting {
                address: canandmessage::cananddevice::types::Setting::CanId,
                value: [value, 0, 0, 0, 0, 0],
                flags: canandmessage::cananddevice::types::SettingFlags {
                    ephemeral: false,
                    synch_hold: false,
                    synch_msg_count: 0,
                },
            }
            .try_into_wrapper(id)
            .map_err(|e| {
                log_error!("Could not serialize id message: {e}");
                fifocore::error::Error::BusWriteFail
            })?;
        msg.0.bus_id = self.bus_id;
        self.fifocore.write_single(&msg)?;
        // If we are setting an id on an arbitrated device, we remove its serial numer from the conflict pool.
        // If we are not, we move the device from the known device pool and leave it to enumeration to pick up the device again.
        let key = DeviceKey::from(FRCCanId(id));
        let should_remove = self.devices.get_mut(&key).map_or(false, |entry| {
            if entry.in_conflict() {
                entry.set_arb_serial_as_diff_id();
                false
            } else {
                true
            }
        });
        if should_remove {
            drop(self.devices.remove(&key));
            self.stale_device = Some(key);
        }

        Ok(())
    }

    pub fn send_fetch_setting(&mut self, id: u32, index: u8) -> Result<(), fifocore::error::Error> {
        let id = FRCCanId(sanitize_id(id));

        let fetch_setting_id = build_frc_can_id(
            id.device_type_code(),
            id.manufacturer_code(),
            canandmessage::cananddevice::MessageIndex::SettingCommand as u16,
            id.device_number(),
        );

        let msg = expand(
            [
                canandmessage::cananddevice::types::SettingCommand::FetchSettingValue as u8,
                index,
            ],
            0,
        );
        let msg = ReduxFIFOMessage::id_data(self.bus_id, fetch_setting_id, msg, 2, 0);
        let key = DeviceKey::from(id);
        if let Some(entry) = self.devices.get_mut(&key) {
            entry.setting_cache_mut().remove_entry(&index);
        }
        self.fifocore.write_single(&msg)?;
        Ok(())
    }

    pub fn send_set_name(&mut self, id: u32, name: &str) -> Result<(), fifocore::error::Error> {
        let id = FRCCanId(sanitize_id(id));

        let set_setting_id = build_frc_can_id(
            id.device_type_code(),
            id.manufacturer_code(),
            canandmessage::cananddevice::MessageIndex::SetSetting as u16,
            id.device_number(),
        );
        let mut name_buf = [0_u8; 18];
        let name_len = name.as_bytes().len().min(name_buf.len());
        name_buf[..name_len].copy_from_slice(&name.as_bytes()[..name_len]);
        let name_indexes = [
            (canandmessage::cananddevice::types::Setting::Name0 as u8, 0_usize),
            (canandmessage::cananddevice::types::Setting::Name1 as u8, 6_usize),
            (canandmessage::cananddevice::types::Setting::Name2 as u8, 12_usize),
        ];

        let key = DeviceKey::from(id);
        for (stg_idx, chunk_start) in name_indexes {
            let mut body = [0_u8; 8];
            body[0] = stg_idx;
            body[1..7].copy_from_slice(&name_buf[chunk_start..chunk_start + 6]);
            let msg = ReduxFIFOMessage::id_data(self.bus_id, set_setting_id, expand(body, 0), 8, 0);
            self.fifocore.write_single(&msg)?;
            if let Some(entry) = self.devices.get_mut(&key) {
                entry.setting_cache_mut().remove_entry(&stg_idx);
            }
        }

        Ok(())
    }

    pub fn send_reboot(&mut self, id: u32, bootloader: bool) -> Result<(), fifocore::error::Error> {
        let id = FRCCanId(sanitize_id(id));
        const BOOT_NORMALLY: rdxota_protocol::otav2::Command = rdxota_protocol::otav2::Command::SysCtl([
            rdxota_protocol::otav2::index::sysctl::BOOT_NORMALLY, 0, 0, 0, 0, 0, 0
        ]);
        const BOOT_TO_DFU: rdxota_protocol::otav2::Command = rdxota_protocol::otav2::Command::SysCtl([
            rdxota_protocol::otav2::index::sysctl::BOOT_TO_DFU, 0, 0, 0, 0, 0, 0
        ]);

        let message_id = build_frc_can_id(
            id.device_type_code(),
            id.manufacturer_code(),
            canandmessage::cananddevice::MessageIndex::OtaToDevice as u16,
            id.device_number(),
        );
        let msg = ReduxFIFOMessage::id_data(self.bus_id, message_id, expand::<_, 8, _>(if bootloader {
            BOOT_TO_DFU.into()
        } else {
            BOOT_NORMALLY.into()
        }, 0), 8, 0);
        self.fifocore.write_single(&msg)?;
        self.devices.remove(&id.into());

        Ok(())
    }

    pub fn setting_cache(&self, id: u32, index: u8) -> Option<FetchSetting> {
        let id = FRCCanId(sanitize_id(id));
        let key = DeviceKey::from(id);
        self.devices
            .get(&key)?
            .setting_cache()
            .get(&index)
            .map(|entry| FetchSetting {
                index,
                data: *entry,
            })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct FetchSetting {
    pub index: u8,
    pub data: [u8; 6],
}

pub async fn bus_session(
    start_gate: tokio::sync::oneshot::Receiver<()>,
    session: Session,
    bus_sessions: Arc<Mutex<FxHashMap<u16, BusState>>>,
) {
    // we need to wait for the bus session map to be populated before the actual logic of this task starts.
    let _ = start_gate.await;

    let bus = session.session().bus_id();
    let mut buffer = session.read_buffer(256);
    let mut interval = tokio::time::interval(Duration::from_millis(5));
    loop {
        interval.tick().await;

        if let Err(e) = session.read_barrier(&mut buffer) {
            log_error!("[ReduxCore] Read session failed: {e}");
            return;
        }
        let mut bus_ses = bus_sessions.lock();
        let Some(state) = bus_ses.get_mut(&bus) else {
            return;
        };
        state.ingest_buffer(&buffer);
        state.poll();
        drop(bus_ses);
    }
}
