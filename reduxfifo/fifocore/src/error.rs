use core::fmt;

use num_enum::{IntoPrimitive, TryFromPrimitive};

macro_rules! defn_error {
    ($(($name:ident, $cname:ident, $int_repr:literal, $msg:literal),)+) => {
        $(
            #[doc = $msg]
            pub const $cname: i32 = $int_repr;
        )+
        #[derive(Clone, Copy, PartialEq, Eq, TryFromPrimitive, IntoPrimitive, serde::Serialize, serde::Deserialize)]
        #[repr(i32)]
        pub enum Error {
            $(
                $name = $cname,
            )+
        }

        impl Error {
            pub fn message(&self) -> &'static str {
                match self {
                    $(
                        Self::$name => $msg,
                    )+
                }
            }

            pub fn cstr_message(&self) -> &'static core::ffi::CStr {
                match self {
                    $(
                        Self::$name => cstr_literal::cstr!($msg),
                    )+
                }
            }
        }

    };
}

#[rustfmt::skip]
defn_error!(
    (Unknown ,              REDUXFIFO_UNKNOWN,                 -1, "Unknown"),
    (NotInitialized,        REDUXFIFO_NOT_INITIALIZED,         -2, "ReduxFIFO not initialized"),
    (NullArgument,          REDUXFIFO_NULL_POINTER_ARGUMENT,   -3, "Null pointer passed as argument"),
    (JavaInvalidByteBuffer, REDUXFIFO_JAVA_INVALID_BYTEBUFFER, -4, "Invalid ByteBuffer passed"),

    (InvalidBus,       REDUXFIFO_INVALID_BUS,        -100, "Invalid bus param string or index"),
    (BusAlreadyOpened, REDUXFIFO_BUS_ALREADY_OPENED, -101, "Bus has already been opened"),
    (MaxBusesOpened,   REDUXFIFO_MAX_BUSES_OPENED,   -102, "No more bus IDs can be allocated"),
    (BusNotSupported,  REDUXFIFO_BUS_NOT_SUPPORTED,  -103, "Bus not supported on this platform"),
    (BusClosed,        REDUXFIFO_BUS_CLOSED,         -104, "Bus closed"),
    (FailedToOpenBus,  REDUXFIFO_FAILED_TO_OPEN_BUS, -105, "Failed to open bus"),
    (BusReadFail,      REDUXFIFO_BUS_READ_FAIL,      -106, "Failed to read bus"),
    (BusWriteFail,     REDUXFIFO_BUS_WRITE_FAIL,     -107, "Failed to write message to bus"),
    (BusBufferFull,    REDUXFIFO_BUS_BUFFER_FULL,    -108, "Bus write buffer is full; retry later"),
    (BusDeviceBusy,    REDUXFIFO_BUS_DEVICE_BUSY,    -109, "Bus device is claimed by another backend (e.g. another USB backend)."),

    (InvalidSessionID,       REDUXFIFO_INVALID_SESSION_ID,        -200, "Invalid session ID"),
    (SessionAlreadyOpened,   REDUXFIFO_SESSION_ALREADY_OPENED,    -201, "Session ID already opened"),
    (MaxSessionsOpened,      REDUXFIFO_MAX_SESSIONS_OPENED,       -202, "Maximum number of sessions opened"),
    (SessionClosed,          REDUXFIFO_SESSION_CLOSED,            -203, "Session closed duriong operation"),
    (MessageReceiveTimeout,  REDUXFIFO_MESSAGE_RECEIVE_TIMEOUT,   -204, "Message receive timeout"),

    (HalCanOpenSessionFail,  REDUXFIFO_HAL_CAN_OPEN_SESSION_FAIL, -301, "HAL_CAN_OpenStreamSession() failed"),
    (UsbClosed,              REDUXFIFO_USB_CLOSED,                -302, "USB transport has closed"),

    (DataTooLong,            REDUXFIFO_DATA_TOO_LONG,             -400, "Data length too long for this transport backend"),
);

impl Error {
    pub fn from_code(code: i32) -> Result<(), Self> {
        if code == REDUXFIFO_OK {
            Ok(())
        } else {
            Err(Self::try_from(code).unwrap_or(Self::Unknown))
        }
    }
}

pub const REDUXFIFO_OK: i32 = 0;

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "ReduxFIFOError {{ {} }}", self.message())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ReduxFIFO Error: \"{}\"!", self.message())
    }
}

impl core::error::Error for Error {}
