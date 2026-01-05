use std::{sync::Arc, time::Duration};

use nusb::{
    DeviceInfo,
    transfer::{ControlIn, ControlType, Recipient},
};
use parking_lot::Mutex;
use rdxusb_protocol::{RdxUsbCtrl, RdxUsbDeviceInfo, RdxUsbPacket};
use rustc_hash::FxHashMap;
use tokio::{
    io::{AsyncReadExt as _, AsyncWriteExt},
    sync::mpsc::error::TryRecvError,
};

use crate::{
    MessageIdBuilder, ReduxFIFOMessage,
    backends::{
        Backend, SessionTable,
        usb::{
            BulkIn, BulkOut, UsbDevice, UsbDeviceId, UsbError, UsbEventLoop, UsbSession,
            UsbSessionState,
        },
    },
    error::Error,
    log_debug, log_error, log_trace,
};

impl From<ReduxFIFOMessage> for RdxUsbPacket {
    fn from(value: ReduxFIFOMessage) -> Self {
        let mut message_id = value.message_id & 0x1fff_ffff;
        if value.device() {
            message_id |= rdxusb_protocol::MESSAGE_ARB_ID_DEVICE;
        }

        if value.rtr() {
            message_id |= rdxusb_protocol::MESSAGE_ARB_ID_RTR;
        }

        if !value.short_id() {
            message_id |= rdxusb_protocol::MESSAGE_ARB_ID_EXT;
        }

        Self {
            message_id,
            channel: 0,
            reserved: 0,
            data_size: value.data_size,
            timestamp_ns: 0,
            data: value.data,
        }
    }
}

impl From<RdxUsbPacket> for ReduxFIFOMessage {
    fn from(value: RdxUsbPacket) -> Self {
        Self {
            message_id: MessageIdBuilder::new(value.message_id)
                .rtr(value.message_id & rdxusb_protocol::MESSAGE_ARB_ID_RTR != 0)
                .short_id(value.message_id & rdxusb_protocol::MESSAGE_ARB_ID_EXT == 0)
                .build(),
            bus_id: value.channel,
            flags: 0,
            data_size: value.data_size,
            timestamp: value.timestamp_ns / 1000,
            data: value.data,
        }
    }
}

async fn rdxusb_loop(
    mut usb_ses: UsbDevice,
    mut tx_msgs: tokio::sync::mpsc::Receiver<(ReduxFIFOMessage, u16)>,
    sessions: Arc<Mutex<FxHashMap<u16, Arc<Mutex<SessionTable<UsbSessionState>>>>>>,
) {
    log_trace!("rdxusb: start new eventloop for {:?}", usb_ses.device_id);
    loop {
        let Ok(device_info) = usb_ses.devinfo().await else {
            return;
        };
        let (tx_ep, rx_ep) = match run_device(device_info).await {
            Ok(d) => d,
            Err(e) => {
                log_error!(
                    "rdxusb: Device open failed for {:?}: {e:?}",
                    usb_ses.device_id
                );
                tokio::time::sleep(Duration::from_millis(100)).await;
                continue;
            }
        };
        log_trace!(
            "rdxusb: device opened successfully: {:?}",
            usb_ses.device_id
        );

        let tx_fut = run_tx(tx_ep, &mut tx_msgs);
        let rx_fut = run_rx(rx_ep, sessions.clone());
        tokio::select! {
            Err(e) = tx_fut => { log_error!("rdxusb: TX closed: {e:?}"); }
            Err(e) = rx_fut => { log_error!("rdxusb: RX closed: {e:?}"); }
        }
    }
}

async fn run_device(device_info: DeviceInfo) -> Result<(BulkOut, BulkIn), UsbError> {
    let Some(iface) = device_info
        .interfaces()
        .find(|iface| iface.class() == 0xff && iface.subclass() == 0x0 && iface.protocol() == 0x0)
    else {
        return Err(UsbError::InterfaceMissing);
    };
    let iface_idx = iface.interface_number();

    let mut handle = Err(UsbError::Other);
    for _ in 0..3 {
        match device_info.open().await {
            Ok(o) => {
                handle = Ok(o);
                break;
            }
            Err(e) => {
                log_error!("rdxusb: Could not open device: {e}");
                handle = Err(UsbError::Nusb(e));
                tokio::time::sleep(Duration::from_millis(10)).await;
                continue;
            }
        }
    }
    let handle = handle?;
    // not all platforms will do this successfully, so this is a best-faith effort.
    handle.detach_kernel_driver(iface_idx).ok();
    let iface = handle.claim_interface(iface_idx).await?;
    let Some(iface_desc) = handle
        .active_configuration()
        .map_err(|_| UsbError::InterfaceMissing)?
        .interface_alt_settings()
        .find(|iface| iface.interface_number() == iface_idx)
    else {
        return Err(UsbError::InterfaceMissing);
    };
    let mut ep_num_out = None;
    let mut ep_num_in = None;
    for ep_desc in iface_desc.endpoints() {
        match ep_desc.direction() {
            nusb::transfer::Direction::Out => ep_num_out.replace(ep_desc.address()),
            nusb::transfer::Direction::In => ep_num_in.replace(ep_desc.address()),
        };
    }
    if ep_num_out.is_none() || ep_num_in.is_none() {
        return Err(UsbError::InterfaceMissing);
    }
    let res = iface
        .control_in(
            ControlIn {
                control_type: ControlType::Vendor,
                recipient: Recipient::Interface,
                request: RdxUsbCtrl::DeviceInfo as u8,
                value: 1,
                index: iface.interface_number() as u16,
                length: core::mem::size_of::<RdxUsbDeviceInfo>() as u16,
            },
            Duration::from_secs(3),
        )
        .await?;
    let rdxusb_info = bytemuck::try_from_bytes::<RdxUsbDeviceInfo>(&res.as_slice())
        .map_err(|_| UsbError::InvalidDevInfo)?;
    if (
        rdxusb_info.protocol_version_major,
        rdxusb_info.protocol_version_minor,
    ) != (2, 0)
    {
        return Err(UsbError::WrongProtocolVersion(2, 0));
    }

    let tx_ep = iface.endpoint(ep_num_out.unwrap())?;
    let rx_ep = iface.endpoint(ep_num_in.unwrap())?;

    Ok((tx_ep, rx_ep))
}

async fn run_tx(
    tx_ep: BulkOut,
    msgs: &mut tokio::sync::mpsc::Receiver<(ReduxFIFOMessage, u16)>,
) -> Result<(), UsbError> {
    let mut writer = tx_ep.writer(64).with_num_transfers(2);
    let mut out_queue = Vec::new();

    loop {
        let Some(pair) = msgs.recv().await else {
            return Ok(());
        };
        let mut pair = Some(pair);
        while let Some((msg, chn)) = pair {
            let mut data: RdxUsbPacket = msg.into();
            data.channel = chn;
            out_queue.extend_from_slice(&bytemuck::bytes_of(&data)[..data.wire_length()]);

            pair = match msgs.try_recv() {
                Ok(p) => Some(p),
                Err(TryRecvError::Disconnected) => {
                    return Ok(());
                }
                Err(TryRecvError::Empty) => None,
            }
        }
        writer.write_all(&out_queue).await?;
        out_queue.clear();
    }
}

async fn run_rx(
    rx_ep: BulkIn,
    sessions: Arc<Mutex<FxHashMap<u16, Arc<Mutex<SessionTable<UsbSessionState>>>>>>,
) -> Result<(), UsbError> {
    let reader = rx_ep.reader(64).with_num_transfers(2);
    let mut buf_reader = tokio::io::BufReader::new(reader);
    let mut packet = [0_u8; 80];

    loop {
        // read the header
        buf_reader.read_exact(&mut packet[..16]).await?;
        let data_length = (packet[7] as usize).min(64);
        // read the rest
        buf_reader
            .read_exact(&mut packet[16..16 + data_length])
            .await?;

        let mut msg: ReduxFIFOMessage = (*RdxUsbPacket::from_buf(&packet)).into();
        msg.timestamp = crate::timebase::now_us() as u64;
        let channel_id = msg.bus_id;

        let meta_ses = sessions.lock();
        let Some(bus) = meta_ses.get(&channel_id) else {
            continue;
        };

        let mut ses_lock = bus.lock();
        // we need to reassign the bus id here to the actually reduxfifo-mapped bus id
        msg.bus_id = ses_lock.bus_id;
        ses_lock.ingest_message(msg);
    }
}

fn split_once<'a>(s: &'a str, d: &str) -> Result<(&'a str, &'a str), Error> {
    s.split_once(d).ok_or(Error::InvalidBus)
}

#[derive(Debug)]
pub struct RdxUsbBackend {
    params: Params,
    handle: Arc<UsbSession>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Params {
    vid: u16,
    pid: u16,
    serial: String,
    channel: u16,
}

impl RdxUsbBackend {
    fn parse_params(s: &str) -> Result<Params, Error> {
        // rdxusb:[chn].[vid].[pid].[serial]
        let (backend_type, backend_args) = split_once(s, ":")?;
        if backend_type != "rdxusb" {
            return Err(Error::BusNotSupported);
        }
        let (channel_str, rest) = split_once(backend_args, ".")?;
        let channel = channel_str.parse::<u16>().map_err(|_| Error::InvalidBus)?;
        let (vid_str, rest) = split_once(rest, ".")?;
        let vid = u16::from_str_radix(vid_str, 16).map_err(|_| Error::InvalidBus)?;
        let (pid_str, serial) = split_once(rest, ".")?;
        let pid = u16::from_str_radix(pid_str, 16).map_err(|_| Error::InvalidBus)?;
        let serial = serial.to_string();

        Ok(Params {
            vid,
            pid,
            serial,
            channel,
        })
    }

    pub fn open(
        bus_id: u16,
        params: &str,
        runtime: tokio::runtime::Handle,
        ses_table: Arc<Mutex<SessionTable<<Self as Backend>::State>>>,
        usb_event_loop: Arc<Mutex<UsbEventLoop>>,
    ) -> Result<Self, crate::error::Error> {
        log_debug!("open rdxusb: {bus_id}");
        let params = match Self::parse_params(params) {
            Ok(p) => p,
            Err(e) => {
                log_error!("Invalid RdxUSB bus string {params}");
                log_error!(
                    "Bus strings are expected for the form `rdxusb:[channel index].[vid in hex].[pid in hex].[usb serial]"
                );
                return Err(e);
            }
        };

        let usb_device_id = UsbDeviceId::new(params.vid, params.pid, params.serial.clone());

        // ok let's open the device, if we need to.
        let handle = {
            log_trace!("rdxusb: request open device");
            let mut eloop = usb_event_loop.lock();
            eloop.open(
                usb_device_id,
                params.channel,
                runtime.clone(),
                ses_table,
                "rdxusb",
                rdxusb_loop,
            )
        };

        // USB device is already claimed by some other backend
        if handle.tag() != "rdxusb" {
            return Err(Error::BusDeviceBusy);
        }

        Ok(Self { params, handle })
    }
}

impl Backend for RdxUsbBackend {
    type State = UsbSessionState;

    fn start_session(
        &mut self,
        _msg_count: u32,
        _config: &crate::ReduxFIFOSessionConfig,
    ) -> Result<Self::State, Error> {
        Ok(UsbSessionState {
            channel: self.params.channel,
        })
    }

    fn write_single(&mut self, msg: &crate::ReduxFIFOMessage) -> Result<(), Error> {
        self.handle
            .msg_tx()
            .try_send((*msg, self.params.channel))
            .map_err(|_| Error::BusBufferFull)
    }

    fn params_match(&self, params: &str) -> bool {
        if let Ok(params) = Self::parse_params(params) {
            params == self.params
        } else {
            false
        }
    }

    fn max_packet_size(&self) -> usize {
        64
    }
}
