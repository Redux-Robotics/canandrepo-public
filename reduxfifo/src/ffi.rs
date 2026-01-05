#![allow(non_snake_case)]
use std::{ffi::CStr, time::Duration};

use crate::INSTANCE;
use crate::log_debug;

use fifocore::{
    ReadBuffer, ReduxFIFOMessage, ReduxFIFOReadBuffer, ReduxFIFOSession, ReduxFIFOSessionConfig,
    ReduxFIFOStatus, ReduxFIFOVersion, ReduxFIFOWriteBuffer, WriteBuffer, error::Error,
};

#[repr(C)]
struct ReduxFIFOReadBufferFFI {
    meta: *mut ReduxFIFOReadBuffer,
    data: *mut ReduxFIFOMessage,
}

#[repr(C)]
struct ReduxFIFOWriteBufferFFI {
    meta: *mut ReduxFIFOWriteBuffer,
    data: *mut ReduxFIFOMessage,
}

/// Returns the version number. This number is unique per version.
///
/// Minor version is bits 0-7
/// Major version is bits 8-15
/// Year is bits 16-30
#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_GetVersion() -> u32 {
    ReduxFIFOVersion::version().serialized()
}

/// Returns a null-terminated UTF-8 error message string.
#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_ErrorMessage(status: i32) -> *const libc::c_char {
    match Error::from_code(status) {
        Ok(()) => c"Ok",
        Err(e) => e.cstr_message(),
    }
    .as_ptr()
}

/// Inits the Redux CANLink server that serves the frontend's websocket and provides CAN messages to the vendordep.
/// This is generally called by the CanandEventLoop in either C++ or Java and doesn't need to be directly called.
/// This function is idempotent and will do nothing if called multiple times.
///
/// Return 0 on success, -1 on already started
#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_StartServer() -> i32 {
    crate::legacy::ReduxCore_InitServer()
}

/// Stops the Redux CANLink server.
/// This is called by CanandEventLoop to stop CANLink.
///
/// Return 0 on success, -1 on already stopped
///
#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_StopServer() -> i32 {
    crate::legacy::ReduxCore_StopServer()
}

/// C ABI open bus
#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_OpenBus(
    bus_address: *const libc::c_char,
    bus_id: *mut u16,
) -> ReduxFIFOStatus {
    if bus_address.is_null() {
        return Err(Error::NullArgument).into();
    }

    let Ok(params) = unsafe { CStr::from_ptr(bus_address) }.to_str() else {
        return Err(Error::InvalidBus).into();
    };
    log_debug!("FFI open bus: {params}");
    INSTANCE
        .open_or_get_bus(params)
        .map(|id| unsafe {
            *bus_id = id;
        })
        .into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_CloseBus(bus_id: u16) -> ReduxFIFOStatus {
    log_debug!("FFI close bus: {bus_id}");
    INSTANCE.close_bus(bus_id).into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_OpenSession(
    bus_id: u16,
    msg_count: u32,
    config: *const ReduxFIFOSessionConfig,
    session_id: *mut ReduxFIFOSession,
) -> ReduxFIFOStatus {
    if config.is_null() || session_id.is_null() {
        return Err(Error::NullArgument).into();
    }

    INSTANCE
        .open_session(bus_id, msg_count, unsafe { config.read() })
        .map(|ses| unsafe {
            *session_id = ses;
        })
        .into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_CloseSession(session: ReduxFIFOSession) -> ReduxFIFOStatus {
    INSTANCE.close_session(session).map(|_| ()).into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_AllocateReadBuffer(
    session: ReduxFIFOSession,
    msg_count: u32,
) -> ReduxFIFOReadBufferFFI {
    let (meta, data, _len) = unsafe { ReadBuffer::new(session, msg_count).into_parts() };
    ReduxFIFOReadBufferFFI { meta, data }
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_FreeReadBuffer(buffer: ReduxFIFOReadBufferFFI) {
    unsafe {
        drop(ReadBuffer::from_parts(buffer.meta, buffer.data));
    }
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_AllocateWriteBuffer(
    bus_id: u16,
    msg_count: u32,
) -> ReduxFIFOWriteBufferFFI {
    let (meta, data, _len) = unsafe {
        WriteBuffer::new(
            bus_id,
            vec![ReduxFIFOMessage::default(); msg_count as usize],
        )
        .into_parts()
    };
    ReduxFIFOWriteBufferFFI { meta, data }
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_FreeWriteBuffer(buffer: ReduxFIFOWriteBufferFFI) {
    unsafe {
        drop(WriteBuffer::from_parts(buffer.meta, buffer.data));
    }
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_ReadBarrier(
    bus_id: u16,
    buffers: *mut ReduxFIFOReadBufferFFI,
    session_count: libc::size_t,
) -> ReduxFIFOStatus {
    if buffers.is_null() {
        return Err(Error::NullArgument).into();
    }
    let meta = unsafe { core::slice::from_raw_parts_mut(buffers, session_count as usize) };
    let mut data: Vec<ReadBuffer> = meta
        .iter()
        .map(|m| unsafe { ReadBuffer::from_parts(m.meta, m.data) })
        .collect();

    INSTANCE.read_barrier(bus_id, &mut data).into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_ReadBarrierMultiBus(
    buffers: *const *const ReduxFIFOReadBufferFFI,
    buffers_lengths: *const libc::size_t,
    buffer_count: libc::size_t,
) -> ReduxFIFOStatus {
    if buffers.is_null() || buffers_lengths.is_null() {
        return Err(Error::NullArgument).into();
    }
    let meta = unsafe { core::slice::from_raw_parts(buffers, buffer_count as usize) };
    let lengths = unsafe { core::slice::from_raw_parts(buffers_lengths, buffer_count as usize) };

    let mut data: Vec<Vec<ReadBuffer>> = meta
        .iter()
        .zip(lengths)
        .map(|(m, &len)| {
            let sub_meta = unsafe { core::slice::from_raw_parts(*m, len) };
            sub_meta
                .iter()
                .map(|m| unsafe { ReadBuffer::from_parts(m.meta, m.data) })
                .collect()
        })
        .collect();

    INSTANCE
        .read_barrier_multibus(data.iter_mut().map(|m| m.as_mut_slice()))
        .into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_WriteBarrier(
    meta: *mut ReduxFIFOWriteBufferFFI,
    session_count: libc::size_t,
) -> ReduxFIFOStatus {
    if meta.is_null() {
        return Err(Error::NullArgument).into();
    }
    let meta = unsafe { core::slice::from_raw_parts_mut(meta, session_count as usize) };
    let mut data: Vec<WriteBuffer> = meta
        .iter()
        .map(|m| unsafe { WriteBuffer::from_parts(m.meta, m.data) })
        .collect();

    INSTANCE.write_barrier(&mut data);
    Ok(()).into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_WriteSingle(msg: *const ReduxFIFOMessage) -> ReduxFIFOStatus {
    unsafe { msg.as_ref() }
        .map_or(Err(Error::NullArgument), |msg| INSTANCE.write_single(msg))
        .into()
}

#[unsafe(no_mangle)]
extern "C" fn ReduxFIFO_WaitForThreshold(
    session: ReduxFIFOSession,
    threshold: u32,
    timeout_ms: u64,
    msg_count: *mut u32,
) -> ReduxFIFOStatus {
    let msg_count = unsafe { msg_count.as_mut() };
    let mut notifier = match INSTANCE.rx_notifier(session) {
        Ok(n) => n,
        Err(e) => {
            return Err(e).into();
        }
    };

    INSTANCE
        .runtime()
        .block_on((async move || match tokio::time::timeout(
            Duration::from_millis(timeout_ms.into()),
            notifier.wait_for(|size| *size > threshold),
        )
        .await
        {
            Ok(Ok(p)) => {
                msg_count.map(|r| {
                    *r = *p;
                });
                drop(p);

                Ok(())
            }
            Ok(Err(_)) => Err(Error::InvalidSessionID),
            Err(_) => Err(Error::MessageReceiveTimeout),
        })())
        .into()
}
