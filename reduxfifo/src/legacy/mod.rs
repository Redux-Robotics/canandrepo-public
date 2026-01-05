//! Legacy Redux driver polyfill

use std::{ffi::CStr, time::Duration};

use parking_lot::{Condvar, Mutex};

use crate::subsystems::repeater::Repeater;
use crate::{INSTANCE, log_debug};
use fifocore::{ReduxFIFOMessage, ReduxFIFOVersion, WriteBuffer};
use tokio::{
    sync::{
        mpsc::{self, Receiver as TokioMPSCReceiver},
        watch,
    },
    task::JoinHandle,
};

mod jni;
mod reduxcore;

pub(crate) static RECEIVER: (Mutex<Option<TokioMPSCReceiver<ReduxFIFOMessage>>>, Condvar) =
    (Mutex::new(None), Condvar::new());
pub(crate) fn put_recv(recv: TokioMPSCReceiver<ReduxFIFOMessage>) {
    let mut recv_state = RECEIVER.0.lock();
    let _ = recv_state.replace(recv);
    RECEIVER.1.notify_all();
}

pub(crate) struct ReduxCoreSession {
    bus_task: JoinHandle<()>,
    #[allow(unused)]
    canlink_task: JoinHandle<()>,
    shutdown: watch::Sender<bool>,
    bus_req: mpsc::Sender<reduxcore::BusRequest>,
}

pub(crate) static REDUXCORE: Mutex<Option<ReduxCoreSession>> = Mutex::new(None);

const REDUXCORE_OK: i32 = 0;
const REDUXCORE_FAIL: i32 = -1;

/// Returns the version number. This number is unique per version.
/// * Minor version is bits 0-7
/// * Major version is bits 8-15
/// * Year is bits 16-30
///
/// @return version number integer
#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_GetVersion() -> i32 {
    ReduxFIFOVersion::version().serialized() as i32
}

/// Inits the Redux CANLink server that serves the frontend's websocket and provides CAN messages to the vendordep.
/// This is generally called by the CanandEventLoop in either C++ or Java and doesn't need to be directly called.
/// This function is idempotent and will do nothing if called multiple times.
///
/// @return the WPIHal bus ID on success, -1 on already started
#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_InitServer() -> i32 {
    env_logger::init_from_env(
        env_logger::Env::new().default_filter_or("debug,jni=off,warp=info,hyper=info,nusb=info"),
    );
    let mut canlink_handle = REDUXCORE.lock();
    if canlink_handle.is_some() {
        -1
    } else {
        log_debug!("ReduxCore Init server");
        let (bus_req, bus_recv) = tokio::sync::mpsc::channel(10);
        let bus_task = INSTANCE
            .runtime()
            .spawn(reduxcore::run_reduxcore(INSTANCE.clone(), bus_recv));

        let (sd_send, sd_recv) = watch::channel(false);
        let canlink_task = INSTANCE
            .runtime()
            .spawn(canandmiddleware::rest_server::run_web_server(
                sd_recv,
                INSTANCE.clone(),
            ));
        //let canlink_task = INSTANCE
        //    .runtime()
        //    .spawn((async |mut sd_recv: watch::Receiver<bool>| { let _ = sd_recv.changed().await; })(sd_recv));
        *canlink_handle = Some(ReduxCoreSession {
            bus_task,
            canlink_task,
            shutdown: sd_send,
            bus_req,
        });

        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_StopServer() -> i32 {
    let mut canlink_handle = REDUXCORE.lock();
    if let Some(hdl) = canlink_handle.take() {
        hdl.bus_task.abort();
        let _ = hdl.shutdown.send(true);
    }
    0
}

///
/// Sends a CAN message to the bus with the specified handle ID.
///
/// Currently only supports bus 0 (the Rio's bus) so it directly calls HAL_CAN_SendMessage and returns the status
///
/// * canBusID - bus id to send to
/// * messageID - message ID to send
/// * data - the data associated with the message
/// * dataSize - the message data size (0-64)
/// @return 0 on success, negative on failure.
#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_EnqueueCANMessage(
    can_bus_id: u16,
    message_id: u32,
    data: *const u8,
    data_size: u8,
) -> i32 {
    let data_slice = unsafe { core::slice::from_raw_parts(data, data_size as usize) };
    let size = (data_size as usize).min(64);
    let mut data_64 = [0_u8; 64];
    data_64[..size].copy_from_slice(&data_slice[..size]);

    let msg = ReduxFIFOMessage::id_data(can_bus_id, message_id, data_64, size as u8, 0);
    let mut ctr = 10;
    loop {
        let result = INSTANCE.write_single(&msg);
        let Err(e) = result else {
            return fifocore::error::REDUXFIFO_OK;
        };

        if e == fifocore::error::Error::BusBufferFull && ctr > 0 {
            std::thread::sleep(Duration::from_millis(10));
            ctr -= 1;
            continue;
        } else {
            return e as i32;
        }
    }
}

/**
 * Sends multiple CAN messages to the bus with the specified handle ID.
 *
 * All messages will be sent to the same bus. This is an intentional limitation.
 *
 * @param[in] messages array of messages to send
 * @param[in] messageCount number of messages to queue
 * @param[out] messagesSent number of messages actually sent
 * @return 0 on success, negative on failure.
*/
#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_BatchEnqueueCANMessages(
    messages: *const ReduxFIFOMessage,
    message_count: usize,
    messages_sent: *mut usize,
) -> i32 {
    let msg_slice = unsafe { core::slice::from_raw_parts(messages, message_count) };

    let Some(msg0) = msg_slice.get(0) else {
        unsafe {
            *messages_sent = 0;
        }
        return 0;
    };
    let bus_id = msg0.bus_id;
    let mut write_buffer = WriteBuffer::new(bus_id, Vec::from(msg_slice));
    INSTANCE.write_barrier(core::array::from_mut(&mut write_buffer));

    unsafe {
        *messages_sent = write_buffer.messages_written() as usize;
    }

    write_buffer
        .status()
        .err()
        .map_or(fifocore::error::REDUXFIFO_OK, i32::from)
}

/**
 * Sends multiple CAN messages to the bus with the specified handle ID.
 *
 * @param[out] messages array of messages to receive into
 * @param[in] messageCount the maximum number of messages to receive
 * @param[out] messagesSent number of messages actually received
 * @return 0 on success, negative on failure.
*/
#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_BatchWaitForCANMessages(
    messages: *mut ReduxFIFOMessage,
    message_count: usize,
    messages_read: *mut usize,
) -> i32 {
    let mut recv = RECEIVER.0.lock();
    // Wait for the receiver to be ready.
    let recv_pipe = loop {
        let Some(recv_pipe) = recv.as_mut() else {
            RECEIVER.1.wait(&mut recv);
            continue;
        };
        break recv_pipe;
    };
    let mut msg_buf = Vec::with_capacity(message_count);
    let read_count = recv_pipe.blocking_recv_many(&mut msg_buf, message_count);
    let messages_slice = unsafe {
        *messages_read = read_count;
        core::slice::from_raw_parts_mut(messages, message_count)
    };
    messages_slice[..read_count].copy_from_slice(&msg_buf[..read_count]);

    if read_count == 0 {
        REDUXCORE_FAIL // the pipe has been closed.
    } else {
        REDUXCORE_OK
    }
}

/**
 * Blocks until a CAN message has been received by CANLink server and writes the result to msgBuf.
 *
 * @param[out] msgBuf message pointer to receive into
 * @return 0 on success, negative on failure. A value of -1 indicates the server has shut down.
*/
#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_WaitForCANMessage(msg_buf: *mut ReduxFIFOMessage) -> i32 {
    let mut recv = RECEIVER.0.lock();
    // Wait for the receiver to be ready.
    let recv_pipe = loop {
        let Some(recv_pipe) = recv.as_mut() else {
            RECEIVER.1.wait(&mut recv);
            continue;
        };
        break recv_pipe;
    };
    match recv_pipe.blocking_recv() {
        Some(msg) => {
            unsafe {
                *msg_buf = msg;
            }
            REDUXCORE_OK
        }
        None => REDUXCORE_FAIL,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_OpenBusById(bus_id: u16) -> i32 {
    let mut canlink_handle = REDUXCORE.lock();
    if let Some(hdl) = canlink_handle.as_mut() {
        let _ = hdl
            .bus_req
            .blocking_send(reduxcore::BusRequest::Open(bus_id));
        bus_id as i32
    } else {
        fifocore::error::Error::NotInitialized as i32
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_OpenBusByString(bus_str: *const libc::c_char) -> i32 {
    if bus_str.is_null() {
        return fifocore::error::Error::NullArgument as i32;
    }

    let bus_string = unsafe { CStr::from_ptr(bus_str) }
        .to_string_lossy()
        .to_string();

    let bus_id = match INSTANCE.open_or_get_bus(&bus_string) {
        Ok(bus_id) => bus_id,
        Err(e) => {
            return e as i32;
        }
    };
    ReduxCore_OpenBusById(bus_id)
}

#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_CloseBus(bus_id: u16) -> i32 {
    let mut canlink_handle = REDUXCORE.lock();
    if let Some(hdl) = canlink_handle.as_mut() {
        let _ = hdl
            .bus_req
            .blocking_send(reduxcore::BusRequest::Close(bus_id));
    }
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_AllocateBuffer(message_count: libc::size_t) -> *mut ReduxFIFOMessage {
    let mut buf = vec![ReduxFIFOMessage::default(); message_count as usize];
    let ptr = buf.as_mut_ptr();
    // i forgor
    core::mem::forget(buf);
    ptr
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReduxCore_DeallocateBuffer(
    buffer: *mut ReduxFIFOMessage,
    message_count: libc::size_t,
) {
    // SAFETY: Jesus TAKE THE WHEEL!!!!!!!!!!!!!!!!!!!!!!!1
    unsafe {
        let length = (message_count as usize) / core::mem::size_of::<ReduxFIFOMessage>();
        drop(Vec::from_raw_parts(buffer, length, length));
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ReduxCore_NewRepeater() -> *mut Repeater {
    Box::into_raw(Box::new(Repeater::new_stopped(INSTANCE.clone())))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReduxCore_UpdateRepeater(
    repeater: *mut Repeater,
    message: *const ReduxFIFOMessage,
    period_ms: u64,
    times: u64,
) {
    unsafe {
        let repeater = Box::from_raw(repeater);
        repeater.update(*message, Duration::from_millis(period_ms), times);
        let _ = Box::into_raw(repeater);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReduxCore_DeallocateRepeater(repeater: *mut Repeater) {
    unsafe {
        let repeater = Box::from_raw(repeater);
        drop(repeater);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReduxCore_OpenLog(log_path: *const libc::c_char, bus_id: u16) -> i32 {
    if log_path.is_null() {
        return fifocore::error::Error::NullArgument as i32;
    }

    let log_path = unsafe { CStr::from_ptr(log_path) }
        .to_string_lossy()
        .into_owned();
    let log_path = std::path::PathBuf::from(log_path);
    match INSTANCE.open_log(log_path, bus_id) {
        Ok(_) => REDUXCORE_OK,
        Err(e) => e as i32,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn ReduxCore_CloseLog(bus_id: u16) -> i32 {
    match INSTANCE.close_log(bus_id) {
        Ok(_) => REDUXCORE_OK,
        Err(e) => e as i32,
    }
}
