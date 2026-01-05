#![allow(non_snake_case)]

/// JNI functions for the java vendordep
/// this file is somehow even harder to look at than the c++ version
use jni::{
    objects::{JByteBuffer, JClass, JObjectArray, JString},
    sys::{jint, jlong, jsize},
    JNIEnv,
};

use crate::{
    error::REDUXFIFO_JAVA_INVALID_BYTEBUFFER,
    ffi::{self, INSTANCE},
    time_us, ReduxFIFOBuffer, ReduxFIFOBufferPointer, ReduxFIFOMessage, ReduxFIFOVersion,
};

//
//#[unsafe(no_mangle)]
//pub extern "system" fn JNI_OnUnload(_vm: JavaVM, _: *mut libc::c_void) {
//    crate::ReduxCore_StopServer();
//}

const REDUXFIFO_EXCEPTION: &str = "com/reduxrobotics/canand/ReduxFIFOJNI$ReduxFIFOException";

pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_getVersion<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    ReduxFIFOVersion::version().serialized() as jint
}

/// Starts the event loop.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_initialize<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    crate::ffi::ReduxFIFO_StartServer()
}

/// Starts the CANLink webserver.
#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_initServer<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    // TODO: don't use the FFI module for this lol
    crate::ffi::ReduxFIFO_StartServer() as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_stopServer<'local>(
    mut _env: JNIEnv<'local>,
    _class: JClass<'local>,
) -> jint {
    // TODO
    crate::ffi::ReduxFIFO_StopServer() as jint
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_openBus<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    bus_address: JString<'local>,
) -> jint {
    let bus_string: String = match env.get_string(&bus_address) {
        Ok(js) => js.into(),
        Err(e) => {
            env.throw_new(
                "java/lang/IllegalArgumentException",
                format!("Could not read bus string: {e}"),
            )
            .ok();
            return -1;
        }
    };
    match INSTANCE.open_or_get_bus(&bus_string) {
        Ok(id) => id as jint,
        Err(err) => {
            env.throw_new(REDUXFIFO_EXCEPTION, format!("Failed to open bus: {err}"))
                .ok();
            -1
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_closeBus<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    bus_id: jint,
) -> jint {
    match INSTANCE.close_bus(bus_id as u16) {
        Ok(_) => 0,
        // TODO: warn?
        Err(err) => {
            env.throw_new(REDUXFIFO_EXCEPTION, format!("Failed to close bus: {err}"))
                .ok();
            err as jint
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_openSession<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    bus_id: jint,
    initial_buffer: JByteBuffer<'local>, // TODO: verify this works later
    filter_id: jint,
    filter_mask: jint,
) -> jint {
    let Ok(initial_ptr) = check_bytebuf_is_valid(&mut env, &initial_buffer) else {
        return REDUXFIFO_JAVA_INVALID_BYTEBUFFER;
    };
    unsafe {
        match INSTANCE.open_session(
            bus_id as u16,
            initial_ptr,
            filter_id as u32,
            filter_mask as u32,
        ) {
            Ok(s) => s.0 as jint,
            Err(err) => {
                env.throw_new(
                    REDUXFIFO_EXCEPTION,
                    format!("Failed to open session: {err}"),
                )
                .ok();
                err as jint
            }
        }
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_closeSession<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    session_id: jint,
) -> jint {
    match INSTANCE.close_session(crate::ReduxFIFOSession(session_id as u32)) {
        Ok(_) => 0,
        Err(e) => e as jint,
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_readBarrier<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    buffers: JObjectArray<'local>,
) -> jlong {
    let Ok(buffers_len) = env.get_array_length(&buffers) else {
        return 0;
    };
    let mut bufs: Vec<ReduxFIFOBufferPointer> = Vec::with_capacity(buffers_len as usize);
    for i in 0..buffers_len {
        let jbuf = match env.get_object_array_element(&buffers, i as jsize) {
            Ok(o) => JByteBuffer::from(o),
            Err(_) => {
                return 0;
            }
        };
        match check_bytebuf_is_valid(&mut env, &jbuf) {
            Ok(b) => {
                bufs.push(b);
            }
            Err(_) => {
                return 0;
            }
        };
    }

    INSTANCE.read_barrier(&bufs);
    time_us() as jlong
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_writeBarrier<'local>(
    mut env: JNIEnv<'local>,
    _class: JClass<'local>,
    buffers: JObjectArray<'local>,
) -> jlong {
    let Ok(buffers_len) = env.get_array_length(&buffers) else {
        return 0;
    };
    let mut bufs: Vec<ReduxFIFOBufferPointer> = Vec::with_capacity(buffers_len as usize);
    for i in 0..buffers_len {
        let jbuf = match env.get_object_array_element(&buffers, i as jsize) {
            Ok(o) => JByteBuffer::from(o),
            Err(_) => {
                return 0;
            }
        };
        match check_bytebuf_is_valid(&mut env, &jbuf) {
            Ok(b) => {
                bufs.push(b);
            }
            Err(_) => {
                return 0;
            }
        };
    }

    INSTANCE.write_barrier(&bufs);
    time_us() as jlong
}

//#[unsafe(no_mangle)]
//pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_allocateBuffer<'local>(mut env: JNIEnv<'local>, _class: JClass<'local>, elements: jint) -> JByteBuffer<'local> {
//    if elements < 0 {
//        env.throw_new("java/lang/IllegalArgumentException", format!("Negative number of elements specified")).ok();
//        return unsafe { JByteBuffer::from_raw(core::ptr::null_mut()) };
//    }
//
//    unsafe {
//        let size = core::mem::size_of::<ReduxFIFOBuffer>() + core::mem::size_of::<ReduxFIFOMessage>() * (elements as usize);
//        let mem = std::alloc::alloc(core::alloc::Layout::from_size_align(size, 4).unwrap());
//        match env.new_direct_byte_buffer(mem, size) {
//            Ok(o) => o,
//            Err(_e) => { JByteBuffer::from_raw(core::ptr::null_mut()) }
//        }
//    }
//}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_reduxrobotics_canand_ReduxFIFOJNI_calcBufferSize<'local>(
    _env: JNIEnv<'local>,
    _class: JClass<'local>,
    n_elements: jint,
) -> jint {
    (core::mem::size_of::<ReduxFIFOBuffer>() as jint)
        + (core::mem::size_of::<ReduxFIFOMessage>() as jint) * n_elements
}

fn check_bytebuf_is_valid<'local>(
    env: &mut JNIEnv<'local>,
    bytebuf: &JByteBuffer<'local>,
) -> Result<ReduxFIFOBufferPointer, jint> {
    let buf_size = match env.get_direct_buffer_capacity(bytebuf) {
        Ok(c) => c,
        Err(e) => {
            env.throw_new(
                "java/lang/IllegalArgumentException",
                format!("Could not get buffer capacity: {e}"),
            )
            .ok();
            return Err(REDUXFIFO_JAVA_INVALID_BYTEBUFFER);
        }
    };

    if buf_size < core::mem::size_of::<ReduxFIFOBuffer>() {
        env.throw_new(
            "java/lang/IllegalArgumentException",
            format!("Buffer is too small to contain 24-byte header"),
        )
        .ok();
        return Err(REDUXFIFO_JAVA_INVALID_BYTEBUFFER);
    }

    let buffer_ptr = match env.get_direct_buffer_address(&bytebuf) {
        Ok(p) => match ReduxFIFOBufferPointer::try_new(p as *mut ReduxFIFOBuffer) {
            Ok(p) => p,
            Err(_) => {
                env.throw_new(
                    "java/lang/NullPointerException",
                    format!("ByteBuffer points to null pointer!"),
                )
                .ok();
                return Err(REDUXFIFO_JAVA_INVALID_BYTEBUFFER);
            }
        },
        Err(e) => {
            env.throw_new(
                "java/lang/IllegalArgumentException",
                format!("Could not get buffer pointer: {e}"),
            )
            .ok();
            return Err(REDUXFIFO_JAVA_INVALID_BYTEBUFFER);
        }
    };
    let claimed_size = buffer_ptr.max_length as usize * core::mem::size_of::<ReduxFIFOMessage>();
    if (buf_size - core::mem::size_of::<ReduxFIFOBuffer>()) < claimed_size {
        env.throw_new(
            "java/lang/IllegalArgumentException",
            format!("Buffer claimed size {claimed_size} is smaller than actual size {buf_size}"),
        )
        .ok();
        return Err(REDUXFIFO_JAVA_INVALID_BYTEBUFFER);
    }
    Ok(buffer_ptr)
}
