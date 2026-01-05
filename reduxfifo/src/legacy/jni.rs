#![allow(non_snake_case)]
//! ReduxCore JNI surface.

use crate::{
    legacy::{
        ReduxCore_AllocateBuffer, ReduxCore_BatchEnqueueCANMessages,
        ReduxCore_BatchWaitForCANMessages, ReduxCore_CloseBus, ReduxCore_CloseLog,
        ReduxCore_DeallocateBuffer, ReduxCore_DeallocateRepeater, ReduxCore_NewRepeater,
        ReduxCore_OpenBusById, ReduxCore_OpenBusByString, ReduxCore_OpenLog,
        ReduxCore_UpdateRepeater, ReduxCore_WaitForCANMessage, ReduxFIFOMessage,
    },
    subsystems::repeater::Repeater,
};
use fifocore::ReduxFIFOVersion;
use jni::{
    JNIEnv,
    objects::{JByteArray, JByteBuffer, JClass, JString},
    sys::{jint, jlong},
};

//
//#[unsafe(no_mangle)]
//pub extern "system" fn JNI_OnUnload(_vm: JavaVM, _: *mut libc::c_void) {
//    crate::ReduxCore_StopServer();
//}

const REDUXJNI_EXCEPTION: &str = "com/reduxrobotics/canand/ReduxJNI$ReduxJNIException";

/// **J**ava **O**n **E**rror **V**omit **E**xception and **R**esult
///
/// Takes a [`Result`] and then throws a Java exception if there's an [`Err`].
/// The result is returned back, but the end user is expected to immediately return from the JNI call.
pub fn joever<T, E: core::error::Error>(
    env: &mut JNIEnv<'_>,
    mut expr: impl FnMut(&mut JNIEnv<'_>) -> Result<T, E>,
) -> Result<T, E> {
    expr(env).map_err(|e| {
        let _ = env.throw_new(REDUXJNI_EXCEPTION, format!("ReduxFIFO Error: {e}"));
        e
    })
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_getDriverVersion<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    ReduxFIFOVersion::version().serialized() as jint
}

/// No-op. Deprecated.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_initialize<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    0
}

/// Starts the CANLink webserver.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_initServer<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    super::ReduxCore_InitServer()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_stopServer<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    super::ReduxCore_StopServer()
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_allocateBuffer<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    count: jint,
) -> JByteBuffer<'local> {
    let count = (count as usize).max(0);
    let ptr = ReduxCore_AllocateBuffer(count) as *mut u8;

    let bbuf = unsafe {
        match env.new_direct_byte_buffer(ptr, count * core::mem::size_of::<ReduxFIFOMessage>()) {
            Ok(bbuf) => bbuf,
            Err(e) => {
                let _ = env.throw_new(REDUXJNI_EXCEPTION, format!("ReduxFIFO Error: {e}"));
                JByteBuffer::default()
            }
        }
    };
    bbuf
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_deallocateBuffer<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    buffer: JByteBuffer<'local>,
) {
    let Ok(capacity) = joever(&mut env, |env| env.get_direct_buffer_capacity(&buffer)) else {
        return;
    };
    if capacity % core::mem::size_of::<ReduxFIFOMessage>() != 0 {
        let _ = env.throw_new(
            REDUXJNI_EXCEPTION,
            "This buffer is of unaligned size!!!! Where did you *get* this thing?",
        );
        return;
    }
    let Ok(buffer_addr) = joever(&mut env, |env| env.get_direct_buffer_address(&buffer)) else {
        return;
    };

    // SAFETY: No.
    unsafe {
        ReduxCore_DeallocateBuffer(buffer_addr as *mut ReduxFIFOMessage, capacity);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_waitForCANMessage<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    buffer: JByteBuffer<'local>, // TODO: verify this works later
) -> jint {
    let Ok(capacity) = joever(&mut env, |env| env.get_direct_buffer_capacity(&buffer)) else {
        return -3;
    };
    if capacity < core::mem::size_of::<ReduxFIFOMessage>() {
        return -3;
    }
    let Ok(buffer_addr) = joever(&mut env, |env| env.get_direct_buffer_address(&buffer)) else {
        return -3;
    };

    if buffer_addr.align_offset(core::mem::align_of::<ReduxFIFOMessage>()) == 0 {
        // happy path
        ReduxCore_WaitForCANMessage(buffer_addr as *mut ReduxFIFOMessage)
    } else {
        // Let's just not support unaligned accesses :thumbs_up:
        -1
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_enqueueCANMessage<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    can_bus_id: jint,
    message_id: jint,
    data: JByteArray<'static>,
) -> jint {
    let Ok(data_len) = joever(&mut env, |env| env.get_array_length(&data)) else {
        return -1;
    };
    let data_len = data_len.clamp(0, 8) as usize;
    let mut buffer = [0i8; 64];

    if !env
        .get_byte_array_region(&data, 0, &mut buffer[..data_len])
        .is_ok()
    {
        return -1;
    };

    super::ReduxCore_EnqueueCANMessage(
        can_bus_id as u16,
        message_id as u32,
        (&buffer).as_ptr() as *const u8,
        data_len as u8,
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_enqueueCANMessageAsLong<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    can_bus_id: jint,
    message_id: jint,
    data: jlong,
    length: jint,
) -> jint {
    let data = data.to_le_bytes();

    let data_len = length.clamp(0, 8) as usize;
    super::ReduxCore_EnqueueCANMessage(
        can_bus_id as u16,
        message_id as u32,
        (&data).as_ptr(),
        data_len as u8,
    )
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_batchEnqueueCANMessages<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    buffer: JByteBuffer<'local>,
    message_count: jint,
) -> jint {
    let message_count = (message_count as usize).max(0);
    let Ok(capacity) = joever(&mut env, |env| env.get_direct_buffer_capacity(&buffer)) else {
        return -3;
    };
    if capacity < core::mem::size_of::<ReduxFIFOMessage>() * message_count {
        return -3;
    }
    let Ok(buffer_addr) = joever(&mut env, |env| env.get_direct_buffer_address(&buffer)) else {
        return -3;
    };

    if buffer_addr.align_offset(core::mem::align_of::<ReduxFIFOMessage>()) != 0 {
        // not aligned. use a real buffer buddy.
        return -4;
    }
    let messages = buffer_addr as *const ReduxFIFOMessage;
    let mut messages_sent = 0;
    let status = ReduxCore_BatchEnqueueCANMessages(messages, message_count, &mut messages_sent);

    if status < 0 {
        status
    } else {
        messages_sent as jint
    }
}
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_batchWaitForCANMessage<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    buffer: JByteBuffer<'local>,
    message_count: jint,
) -> jint {
    let message_count = (message_count as usize).max(0);
    let Ok(capacity) = joever(&mut env, |env| env.get_direct_buffer_capacity(&buffer)) else {
        return -3;
    };
    if capacity < core::mem::size_of::<ReduxFIFOMessage>() * message_count {
        return -3;
    }
    let Ok(buffer_addr) = joever(&mut env, |env| env.get_direct_buffer_address(&buffer)) else {
        return -3;
    };

    if buffer_addr.align_offset(core::mem::align_of::<ReduxFIFOMessage>()) != 0 {
        // not aligned. use a real buffer buddy.
        return -4;
    }
    let messages = buffer_addr as *mut ReduxFIFOMessage;
    let mut messages_read = 0;

    let status = ReduxCore_BatchWaitForCANMessages(messages, message_count, &mut messages_read);
    if status < 0 {
        status
    } else {
        messages_read as jint
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_openBusByString<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    bus_address: JString<'local>,
) -> jint {
    let bus_str = match env.get_string(&bus_address) {
        Ok(js) => js,
        Err(e) => {
            env.throw_new(
                "java/lang/IllegalArgumentException",
                format!("Could not read bus string: {e}"),
            )
            .ok();
            return -1;
        }
    };
    ReduxCore_OpenBusByString(bus_str.as_ptr()) as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_openBusById<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    bus_id: jint,
) -> jint {
    ReduxCore_OpenBusById(bus_id as u16) as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_closeBus<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    bus_id: jint,
) -> jint {
    ReduxCore_CloseBus(bus_id as u16) as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_newRepeater<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jlong {
    (ReduxCore_NewRepeater() as usize) as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_updateRepeater<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    repeater_handle: jlong,
    can_bus_id: jint,
    message_id: jint,
    data: jlong,
    length: jint,
    period_ms: jint,
    times: jint,
) {
    let message = ReduxFIFOMessage::id_data(
        can_bus_id as u16,
        message_id as u32,
        right_pad(data.to_le_bytes(), 0),
        length as u8,
        0,
    );

    unsafe {
        ReduxCore_UpdateRepeater(
            (repeater_handle as usize) as *mut Repeater,
            &message,
            period_ms as u64,
            times as u64,
        );
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_deallocateRepeater<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    repeater_handle: jlong,
) {
    unsafe {
        ReduxCore_DeallocateRepeater((repeater_handle as usize) as *mut Repeater);
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_openLog<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    log_path: JString<'local>,
    bus_id: jint,
) -> jint {
    unsafe {
        let log_str = match env.get_string(&log_path) {
            Ok(js) => js,
            Err(e) => {
                env.throw_new(
                    "java/lang/IllegalArgumentException",
                    format!("Could not read bus string: {e}"),
                )
                .ok();
                return -1;
            }
        };
        ReduxCore_OpenLog(log_str.as_ptr(), bus_id as u16)
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxJNI_closeLog<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    bus_id: jint,
) -> jint {
    unsafe { ReduxCore_CloseLog(bus_id as u16) }
}

const fn right_pad<V: Copy, const S: usize, const T: usize>(a: [V; S], pad: V) -> [V; T] {
    assert!(S <= T);
    let mut value = [pad; T];
    let mut idx = 0_usize;
    while idx < S {
        value[idx] = a[idx];
        idx += 1;
    }
    while idx < T {
        value[idx] = pad;
        idx += 1;
    }
    value
}
