//! This is the primary top-level driver.

// Contains definitions for the Java Native Interface API surface.
//#[cfg(feature = "jni")]
//pub mod jni;

/// Contains definitions for the extern C API surface.
///
/// Rust code should _not_ use these interfaces!
#[cfg(feature = "ffi")]
mod ffi;

/// Native-acceleration for specific vendordep-facing tasks
pub mod subsystems;

#[cfg(feature = "legacy-driver")]
pub mod legacy;

mod log;

//mod canlink;
pub(crate) use crate::log::*;
use fifocore::FIFOCore;

#[cfg(feature = "singleton")]
static RUNTIME: std::sync::LazyLock<tokio::runtime::Runtime> = std::sync::LazyLock::new(|| {
    #[cfg(feature = "tokio-console")]
    console_subscriber::ConsoleLayer::builder()
        .with_default_env()
        .server_addr((std::net::Ipv4Addr::UNSPECIFIED, 6669))
        .init();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .thread_name("ReduxFIFO")
        .build()
        .expect("could not start ReduxFIFO")
});

#[cfg(feature = "singleton")]
pub static INSTANCE: std::sync::LazyLock<FIFOCore> =
    std::sync::LazyLock::new(|| FIFOCore::new(RUNTIME.handle().clone()));
