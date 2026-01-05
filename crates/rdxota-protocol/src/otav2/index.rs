/// Indexes for the otav2 protocol.
/// This is intended to be the Canonical List Of Indexes.
///

pub const OTA_VERSION: u8 = 2;
pub const FIRMWARE_SLOT: u8 = 0;

pub mod ack {
    pub const OK: u8 = 0;
    pub const TRANSFER_START: u8 = 1;
    pub const CHUNK_VERIFIED: u8 = 2;
    pub const CHUNK_COMMITTED: u8 = 3;
    pub const CHUNK_CLEARED: u8 = 4;
    pub const UNKNOWN: u8 = 0;
}

pub mod nack {
    pub const INVALID_ARGUMENT: u8 = 0;
    pub const INVALID_FILE_INDEX: u8 = 1;
    pub const OPERATION_ABORTED: u8 = 2;
    pub const DEVICE_BUSY: u8 = 3;
    pub const ACCESS_DENIED: u8 = 4;

    pub const CHUNK_CRC32_FAIL: u8 = 16;
    pub const COMMIT_FAIL: u8 = 17;
    pub const BUFFER_OVERRUN: u8 = 18;

    pub const UNKNOWN_OTA: u8 = 32;
    pub const HEADER_MAGIC_FAIL: u8 = 33;
    pub const HEADER_VERSION_FAIL: u8 = 34;
    pub const HEADER_PRODUCT_MISMATCH: u8 = 35;
    pub const HEADER_ECIES_KEY_SIG_FAIL: u8 = 36;
    pub const HEADER_HMAC_FAIL: u8 = 37;

    pub const BLOCK_HEADER_MAGIC_FAIL: u8 = 38;
    pub const BLOCK_HEADER_HMAC_FAIL: u8 = 39;
    pub const BLOCK_HEADER_INVALID: u8 = 40;

    pub const DATA_ADDRESS_INVALID: u8 = 41;
    pub const DATA_INVALID: u8 = 42;

    pub const ERASE_FAIL: u8 = 43;
    pub const FLASH_FAIL: u8 = 44;
    pub const FINAL_VERIFICATION_FAILURE: u8 = 45;
    pub const NOT_DONE: u8 = 46;

    pub const UNKNOWN: u8 = 0xff;
}

pub mod ctrl {
    pub const VERSION: u8 = 0;
    pub const STAT: u8 = 1;
    pub const UPLOAD: u8 = 2;
    pub const DOWNLOAD: u8 = 3;

    pub const SYS_CTL: u8 = 4;
    pub const CHALLENGE: u8 = 5;
    pub const RESPONSE: u8 = 6;
    pub const DEVICE_STATE: u8 = 7;

    pub const ACK: u8 = 16;
    pub const NACK: u8 = 17;
    pub const CHUNK_SIZE: u8 = 18;
    pub const VERIFY_CHUNK: u8 = 19;
    pub const FINISH: u8 = 20;
    pub const ABORT: u8 = 21;
    pub const TELL: u8 = 22;
    pub const COMMIT_CHUNK: u8 = 23;
    pub const CLEAR_CHUNK: u8 = 24;
}

// first byte of sysctl command
pub mod sysctl {
    pub const BOOT_NORMALLY: u8 = 0xf5;
    pub const BOOT_TO_DFU: u8 = 0xf8;
    pub const BURN_SERIAL: u8 = 0xb5;
}
