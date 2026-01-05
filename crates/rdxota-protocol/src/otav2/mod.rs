pub mod index;
use index::*;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ack {
    Ok,
    TransferStart(u32),
    ChunkVerified(u32),
    ChunkCommitted(u32),
    ChunkCleared(u32),
    Unknown,
}

impl From<[u8; 8]> for Ack {
    fn from(value: [u8; 8]) -> Self {
        match value[1] {
            ack::OK => Ack::Ok,
            ack::TRANSFER_START => {
                Ack::TransferStart(u32::from_le_bytes(value[2..6].try_into().unwrap()))
            }
            ack::CHUNK_VERIFIED => {
                Ack::ChunkVerified(u32::from_le_bytes(value[2..6].try_into().unwrap()))
            }
            ack::CHUNK_COMMITTED => {
                Ack::ChunkCommitted(u32::from_le_bytes(value[2..6].try_into().unwrap()))
            }
            ack::CHUNK_CLEARED => {
                Ack::ChunkCleared(u32::from_le_bytes(value[2..6].try_into().unwrap()))
            }
            _ => Ack::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Nack {
    InvalidArgument = nack::INVALID_ARGUMENT,
    InvalidFileIndex = nack::INVALID_FILE_INDEX,
    OperationAborted = nack::OPERATION_ABORTED,
    DeviceBusy = nack::DEVICE_BUSY,
    AccessDenied = nack::ACCESS_DENIED,

    ChunkCRC32Fail = nack::CHUNK_CRC32_FAIL,
    CommitFail = nack::COMMIT_FAIL,
    BufferOverrun = nack::BUFFER_OVERRUN,

    UnknownOTA = nack::UNKNOWN_OTA,
    HeaderMagicFail = nack::HEADER_MAGIC_FAIL,
    HeaderVersionFail = nack::HEADER_VERSION_FAIL,
    HeaderProductMismatch = nack::HEADER_PRODUCT_MISMATCH,
    HeaderEciesKeySigFail = nack::HEADER_ECIES_KEY_SIG_FAIL,
    HeaderHmacFail = nack::HEADER_HMAC_FAIL,

    BlockHeaderMagicFail = nack::BLOCK_HEADER_MAGIC_FAIL,
    BlockHeaderHmacFail = nack::BLOCK_HEADER_HMAC_FAIL,
    BlockHeaderInvalid = nack::BLOCK_HEADER_INVALID,

    DataAddressInvalid = nack::DATA_ADDRESS_INVALID,
    DataInvalid = nack::DATA_INVALID,

    EraseFail = nack::ERASE_FAIL,
    FlashFail = nack::FLASH_FAIL,
    FinalVerificationFailure = nack::FINAL_VERIFICATION_FAILURE,
    NotDone = nack::NOT_DONE,

    Unknown = 0xff,
}

impl From<u8> for Nack {
    fn from(value: u8) -> Self {
        match value {
            nack::INVALID_ARGUMENT => Nack::InvalidArgument,
            nack::INVALID_FILE_INDEX => Nack::InvalidFileIndex,
            nack::OPERATION_ABORTED => Nack::OperationAborted,
            nack::DEVICE_BUSY => Nack::DeviceBusy,
            nack::ACCESS_DENIED => Nack::AccessDenied,

            nack::CHUNK_CRC32_FAIL => Nack::ChunkCRC32Fail,
            nack::COMMIT_FAIL => Nack::CommitFail,
            nack::BUFFER_OVERRUN => Nack::BufferOverrun,

            nack::UNKNOWN_OTA => Nack::UnknownOTA,
            nack::HEADER_MAGIC_FAIL => Nack::HeaderMagicFail,
            nack::HEADER_VERSION_FAIL => Nack::HeaderVersionFail,
            nack::HEADER_PRODUCT_MISMATCH => Nack::HeaderProductMismatch,
            nack::HEADER_ECIES_KEY_SIG_FAIL => Nack::HeaderEciesKeySigFail,
            nack::HEADER_HMAC_FAIL => Nack::HeaderHmacFail,

            nack::BLOCK_HEADER_MAGIC_FAIL => Nack::BlockHeaderMagicFail,
            nack::BLOCK_HEADER_HMAC_FAIL => Nack::BlockHeaderHmacFail,
            nack::BLOCK_HEADER_INVALID => Nack::BlockHeaderInvalid,

            nack::DATA_ADDRESS_INVALID => Nack::DataAddressInvalid,
            nack::DATA_INVALID => Nack::DataInvalid,

            nack::ERASE_FAIL => Nack::EraseFail,
            nack::FLASH_FAIL => Nack::FlashFail,
            nack::FINAL_VERIFICATION_FAILURE => Nack::FinalVerificationFailure,
            nack::NOT_DONE => Nack::NotDone,

            _ => Nack::Unknown,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    Version,     // 0
    Stat(u16),   // 1
    Upload(u16), // 2

    SysCtl([u8; 7]), // 4
    DeviceState,     // 7
    Ack(Ack),        // 16
    Nack(Nack),      // 17

    ChunkSize(u32),   // 18
    VerifyChunk(u32), // 19
    Finish,           // 20
    Abort,            // 21
    Tell,             // 22
    CommitChunk(u32), // 23
    ClearChunk(u32),  // 24
}

impl From<Command> for [u8; 8] {
    fn from(cmd: Command) -> Self {
        let mut p = [0u8; 8];
        match cmd {
            Command::Version => {
                p[0] = ctrl::VERSION;
            }
            Command::Stat(file_idx) => {
                p[0] = ctrl::STAT;
                p[1..3].copy_from_slice(&file_idx.to_le_bytes());
            }
            Command::Upload(file_idx) => {
                p[0] = ctrl::UPLOAD;
                p[1..3].copy_from_slice(&file_idx.to_le_bytes());
            }
            Command::SysCtl(data) => {
                p[0] = ctrl::SYS_CTL;
                p[1..8].copy_from_slice(&data);
            }
            Command::DeviceState => {
                p[0] = ctrl::DEVICE_STATE;
            }
            Command::Ack(a) => {
                p[0] = ctrl::ACK;
                match a {
                    Ack::Ok => {
                        p[1] = ack::OK;
                    }
                    Ack::TransferStart(sz) => {
                        p[1] = ack::TRANSFER_START;
                        p[2..6].copy_from_slice(&sz.to_le_bytes());
                    }
                    Ack::ChunkVerified(n) => {
                        p[1] = ack::CHUNK_VERIFIED;
                        p[2..6].copy_from_slice(&n.to_le_bytes());
                    }
                    Ack::ChunkCommitted(n) => {
                        p[1] = ack::CHUNK_COMMITTED;
                        p[2..6].copy_from_slice(&n.to_le_bytes());
                    }
                    Ack::Unknown => {}
                    Ack::ChunkCleared(n) => {
                        p[1] = ack::CHUNK_CLEARED;
                        p[2..6].copy_from_slice(&n.to_le_bytes());
                    }
                }
            }
            Command::Nack(n) => {
                p[0] = ctrl::NACK;
                p[1] = n as u8;
            }
            Command::ChunkSize(sz) => {
                p[0] = ctrl::CHUNK_SIZE;
                p[1..5].copy_from_slice(&sz.to_le_bytes());
            }
            Command::VerifyChunk(crc) => {
                p[0] = ctrl::VERIFY_CHUNK;
                p[1..5].copy_from_slice(&crc.to_le_bytes());
            }
            Command::CommitChunk(n) => {
                p[0] = ctrl::COMMIT_CHUNK;
                p[1..5].copy_from_slice(&n.to_le_bytes());
            }
            Command::ClearChunk(n) => {
                p[0] = ctrl::CLEAR_CHUNK;
                p[1..5].copy_from_slice(&n.to_le_bytes());
            }
            Command::Finish => {
                p[0] = ctrl::FINISH;
            }
            Command::Abort => {
                p[0] = ctrl::ABORT;
            }
            Command::Tell => {
                p[0] = ctrl::TELL;
            }
        }
        p
    }
}

impl TryFrom<[u8; 8]> for Command {
    type Error = ();
    fn try_from(value: [u8; 8]) -> Result<Self, ()> {
        Ok(match value[0] {
            ctrl::VERSION => Command::Version,
            ctrl::STAT => Command::Stat(u16::from_le_bytes(value[1..3].try_into().unwrap())),
            ctrl::UPLOAD => Command::Upload(u16::from_le_bytes(value[1..3].try_into().unwrap())),
            ctrl::SYS_CTL => Command::SysCtl(value[1..8].try_into().unwrap()),
            ctrl::DEVICE_STATE => Command::DeviceState,
            ctrl::ACK => Command::Ack(Ack::from(value)),
            ctrl::NACK => Command::Nack(Nack::from(value[1])),
            ctrl::CHUNK_SIZE => {
                Command::ChunkSize(u32::from_le_bytes(value[1..5].try_into().unwrap()))
            }
            ctrl::VERIFY_CHUNK => {
                Command::VerifyChunk(u32::from_le_bytes(value[1..5].try_into().unwrap()))
            }
            ctrl::FINISH => Command::Finish,
            ctrl::ABORT => Command::Abort,
            ctrl::TELL => Command::Tell,
            ctrl::COMMIT_CHUNK => {
                Command::CommitChunk(u32::from_le_bytes(value[1..5].try_into().unwrap()))
            }
            ctrl::CLEAR_CHUNK => {
                Command::ClearChunk(u32::from_le_bytes(value[1..5].try_into().unwrap()))
            }
            _ => {
                return Err(());
            }
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Response {
    Version(u8),
    Stat(Stat),
    //Upload(u16),

    //SysCtl([u8; 7]),
    DeviceState([u8; 7]),
    Ack(Ack),
    Nack(Nack),

    ChunkSize(u32),
    VerifyChunk(u32),
    Tell(u32),

    Unknown([u8; 8]),
}

impl From<Response> for [u8; 8] {
    fn from(value: Response) -> Self {
        match value {
            Response::Version(v) => [ctrl::VERSION, v, 0, 0, 0, 0, 0, 0],
            Response::Stat(s) => s.to_bytes(),
            Response::DeviceState(s) => {
                let mut v = [0u8; 8];
                v[0] = ctrl::DEVICE_STATE;
                v[1..7].copy_from_slice(&s);
                v
            }
            Response::Ack(a) => {
                let mut v = [0u8; 8];
                v[0] = ctrl::ACK;
                v[1] = match a {
                    Ack::Ok => ack::OK,
                    Ack::TransferStart(s) => {
                        v[2..6].copy_from_slice(&s.to_le_bytes());
                        ack::TRANSFER_START
                    }
                    Ack::ChunkVerified(n) => {
                        v[2..6].copy_from_slice(&n.to_le_bytes());
                        ack::CHUNK_VERIFIED
                    }
                    Ack::ChunkCommitted(n) => {
                        v[2..6].copy_from_slice(&n.to_le_bytes());
                        ack::CHUNK_COMMITTED
                    }
                    Ack::ChunkCleared(n) => {
                        v[2..6].copy_from_slice(&n.to_le_bytes());
                        ack::CHUNK_CLEARED
                    }
                    Ack::Unknown => 0xff,
                };
                v
            }
            Response::Nack(n) => [ctrl::NACK, n as u8, 0, 0, 0, 0, 0, 0],
            Response::ChunkSize(s) => {
                let mut v = [0u8; 8];
                v[0] = ctrl::CHUNK_SIZE;
                v[1..5].copy_from_slice(&s.to_le_bytes());
                v
            }
            Response::VerifyChunk(s) => {
                let mut v = [0u8; 8];
                v[0] = ctrl::VERIFY_CHUNK;
                v[1..5].copy_from_slice(&s.to_le_bytes());
                v
            }
            Response::Tell(s) => {
                let mut v = [0u8; 8];
                v[0] = ctrl::TELL;
                v[1..5].copy_from_slice(&s.to_le_bytes());
                v
            }
            Response::Unknown(s) => s,
        }
    }
}

impl From<[u8; 8]> for Response {
    fn from(value: [u8; 8]) -> Self {
        match value[0] {
            ctrl::VERSION => Self::Version(value[1]),
            ctrl::STAT => Self::Stat(Stat::from(value)),
            ctrl::DEVICE_STATE => Self::DeviceState(value[1..].try_into().unwrap()),
            ctrl::ACK => Self::Ack(Ack::from(value)),
            ctrl::NACK => Self::Nack(Nack::from(value[1])),

            ctrl::CHUNK_SIZE => {
                Self::ChunkSize(u32::from_le_bytes(value[1..5].try_into().unwrap()))
            }
            ctrl::VERIFY_CHUNK => {
                Self::VerifyChunk(u32::from_le_bytes(value[1..5].try_into().unwrap()))
            }
            ctrl::TELL => Self::Tell(u32::from_le_bytes(value[1..5].try_into().unwrap())),

            _ => Self::Unknown(value),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stat {
    pub file_idx: u16,
    pub inode_exists: bool,
    pub inode_readable: bool,
    pub inode_writeable: bool,
    pub inode_executable: bool,
    pub inode_auth: u8,
    pub requires_dfu: bool,
    pub size: u32,
}

impl Stat {
    pub const fn to_bytes(&self) -> [u8; 8] {
        let mut buf = [0u8; 8];
        buf[0] = ctrl::STAT;
        buf[1] = self.file_idx as u8;
        buf[2] = (self.file_idx >> 8) as u8;
        buf[3] = (self.inode_exists as u8)
            | ((self.inode_readable as u8) << 1)
            | ((self.inode_writeable as u8) << 2)
            | ((self.inode_executable as u8) << 3)
            | ((self.inode_auth & 0b111) << 4)
            | ((self.requires_dfu as u8) << 7);

        buf[5] = self.size as u8;
        buf[6] = (self.size >> 8) as u8;
        buf[7] = (self.size >> 16) as u8;

        buf
    }
}

impl From<Stat> for [u8; 8] {
    fn from(value: Stat) -> Self {
        value.to_bytes()
    }
}

impl From<[u8; 8]> for Stat {
    fn from(value: [u8; 8]) -> Self {
        Self {
            file_idx: u16::from_le_bytes(value[1..3].try_into().unwrap()),
            inode_exists: (value[3] & 1) != 0,
            inode_readable: (value[3] & 0b10) != 0,
            inode_writeable: (value[3] & 0b100) != 0,
            inode_executable: (value[3] & 0b1000) != 0,
            inode_auth: (value[3] >> 4),
            requires_dfu: (value[3] >> 7) != 0,
            size: (value[5] as u32) | ((value[6] as u32) << 8) | ((value[7] as u32) << 16),
        }
    }
}
