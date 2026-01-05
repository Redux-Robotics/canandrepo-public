use std::{
    sync::{Arc, Weak},
    time::Duration,
};

use futures::StreamExt;
use nusb::{DeviceInfo, Endpoint, hotplug::HotplugEvent};
use parking_lot::Mutex;
use rustc_hash::FxHashMap;
use tokio::{sync::watch, task::JoinHandle};

use crate::{ReduxFIFOMessage, backends::SessionTable, error::Error, log_trace};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct UsbDeviceId {
    pub vid: u16,
    pub pid: u16,
    // Serial number of the device.
    // This is mandatory to disambiguate devices.
    pub serial_numer: String,
}

impl UsbDeviceId {
    pub const fn new(vid: u16, pid: u16, serial: String) -> Self {
        Self {
            vid,
            pid,
            serial_numer: serial,
        }
    }

    pub fn matches_devinfo(&self, info: &DeviceInfo) -> bool {
        self.vid == info.vendor_id()
            && self.pid == info.product_id()
            && (info
                .serial_number()
                .map_or(false, |ins| self.serial_numer == ins))
    }
}

/// USB session handle runtime data
pub struct UsbDevice {
    pub device_id: UsbDeviceId,
    pub devinfo_watch: watch::Receiver<Option<DeviceInfo>>,
}

impl UsbDevice {
    pub async fn devinfo(&mut self) -> Result<DeviceInfo, Error> {
        if let Some(info) = self.rescan().await {
            return Ok(info);
        }
        loop {
            match tokio::time::timeout(Duration::from_secs(1), self.devinfo_watch.changed()).await {
                Ok(Ok(())) => match self.devinfo_watch.borrow_and_update().clone() {
                    Some(d) => {
                        return Ok(d);
                    }
                    None => {
                        continue;
                    }
                },
                Ok(Err(_)) => {
                    return Err(Error::UsbClosed);
                }
                Err(_) => {
                    if let Some(devinfo) = self.rescan().await {
                        return Ok(devinfo);
                    }
                    continue;
                }
            }
        }
    }

    async fn rescan(&self) -> Option<DeviceInfo> {
        log::trace!(target: "reduxfifo::usb", "Scan devices triggered");
        if let Ok(device_iter) = nusb::list_devices().await {
            for device_info in device_iter {
                if self.device_id.matches_devinfo(&device_info) {
                    log::trace!(target: "reduxfifo::usb", "Found device: {device_info:?}");
                    return Some(device_info);
                }
            }
        };
        None
    }
}

pub(crate) type BulkOut = Endpoint<nusb::transfer::Bulk, nusb::transfer::Out>;
pub(crate) type BulkIn = Endpoint<nusb::transfer::Bulk, nusb::transfer::In>;
pub(crate) type Sessions = Arc<Mutex<FxHashMap<u16, Arc<Mutex<SessionTable<UsbSessionState>>>>>>;
type TxSender = tokio::sync::mpsc::Sender<(ReduxFIFOMessage, u16)>;
type TxReceiver = tokio::sync::mpsc::Receiver<(ReduxFIFOMessage, u16)>;

/// This is always gonna live in an Arc of some sort.
#[derive(Debug)]
pub struct UsbSession {
    device_id: UsbDeviceId,
    devinfo_sender: watch::Sender<Option<DeviceInfo>>,
    msg_tx: TxSender,
    task_handle: JoinHandle<()>,
    tag: String,
    meta_sessions: Sessions,
}

impl UsbSession {
    pub fn device_id_matches(&self, other: &UsbDeviceId) -> bool {
        &self.device_id == other
    }

    pub fn msg_tx(&self) -> &TxSender {
        &self.msg_tx
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }
}

impl Drop for UsbSession {
    fn drop(&mut self) {
        self.task_handle.abort();
    }
}

/// This holds all the devinfo sessions.
/// Dropping the [`UsbSession`] will stop hotplug watching for that device.
#[derive(Debug)]
pub struct UsbEventLoop {
    pub devices: Vec<Weak<UsbSession>>,
}

impl UsbEventLoop {
    pub const fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }

    /// Indicates that a USB device is to be watched for new or existing connections.
    pub fn open<
        R: Future<Output = ()> + Send + 'static,
        F: FnOnce(UsbDevice, TxReceiver, Sessions) -> R,
    >(
        &mut self,
        device_id: UsbDeviceId,
        channel_id: u16,
        runtime: tokio::runtime::Handle,
        sessions: Arc<Mutex<SessionTable<UsbSessionState>>>,
        tag: &str,
        f: F,
    ) -> Arc<UsbSession> {
        let mut ret = None;
        self.devices.retain(|ses| match ses.upgrade() {
            Some(ses) => {
                if ses.device_id_matches(&device_id) {
                    ret = Some(ses);
                }
                true
            }
            None => false,
        });
        if let Some(ses) = ret {
            log_trace!("rdxusb: {device_id:?} already opened on another channel!");
            let mut meta_ses = ses.meta_sessions.lock();
            meta_ses.insert(channel_id, sessions);
            drop(meta_ses);
            return ses;
        }

        log_trace!("rdxusb: create new session for {device_id:?}");
        let (send, recv) = watch::channel(None);
        let device = UsbDevice {
            device_id: device_id.clone(),
            devinfo_watch: recv,
        };
        let (tx_send, tx_recv) = tokio::sync::mpsc::channel(128);

        let mut meta_sessions = FxHashMap::default();
        meta_sessions.insert(channel_id, sessions);
        let meta_sessions = Arc::new(Mutex::new(meta_sessions));

        let ses = Arc::new(UsbSession {
            device_id,
            devinfo_sender: send,
            task_handle: runtime.spawn(f(device, tx_recv, meta_sessions.clone())),
            msg_tx: tx_send,
            tag: tag.to_string(),
            meta_sessions,
        });
        self.devices.push(Arc::downgrade(&ses));
        ses
    }

    /// Hotplug task.
    pub async fn hotplug(event_loop: Arc<Mutex<Self>>) {
        if let Ok(devices) = nusb::list_devices().await {
            for dev in devices {
                log_trace!("Found USB device: {dev:?}");
            }
        }

        let mut hotplug_watcher = nusb::watch_devices().expect("Could not start hotplug watcher!");
        log::trace!(target: "reduxfifo::usb", "Started USB hotplug watcher");
        while let Some(event) = hotplug_watcher.next().await {
            match event {
                HotplugEvent::Connected(device_info) => {
                    log::debug!(target: "reduxfifo::usb", "Device connected: {device_info:?}");
                    let mut eloop = event_loop.lock();
                    for maybe_device in &eloop.devices {
                        if let Some(dev) = maybe_device.upgrade()
                            && dev.device_id.matches_devinfo(&device_info)
                        {
                            dev.devinfo_sender.send_replace(Some(device_info.clone()));
                        }
                    }
                    // garbage collect.
                    eloop.devices.retain(|ses| ses.upgrade().is_some());
                }
                HotplugEvent::Disconnected(device_id) => {
                    log::debug!(target: "reduxfifo::usb", "Device disconnected: {device_id:?}");
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UsbSessionState {
    pub channel: u16,
}

impl UsbSessionState {
    pub fn new(channel: u16) -> Self {
        Self { channel }
    }
}

#[allow(unused)]
#[derive(Debug)]
pub enum UsbError {
    InterfaceMissing,
    Nusb(nusb::Error),
    NusbXfer(nusb::transfer::TransferError),
    IoError(std::io::Error),
    WrongProtocolVersion(u16, u16),
    InvalidDevInfo,
    Other,
}

impl From<nusb::Error> for UsbError {
    fn from(value: nusb::Error) -> Self {
        Self::Nusb(value)
    }
}

impl From<nusb::transfer::TransferError> for UsbError {
    fn from(value: nusb::transfer::TransferError) -> Self {
        Self::NusbXfer(value)
    }
}

impl From<std::io::Error> for UsbError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError(value)
    }
}
