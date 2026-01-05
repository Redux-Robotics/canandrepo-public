//! Serial numbers for Redux products.
//!
//! ## Serial..."numer"???
//!
//! Calling it a `numer` is a long-running in-joke, and is an effective way of
//! distinguishing from other types of serial code.
#![no_std]

use num_enum::{FromPrimitive, IntoPrimitive, TryFromPrimitive};
use rdxcrc::crc4itu_nibble_reverse;

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, TryFromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum LifecycleFlag {
    Mule = 0,
    Prototype = 1,
    Preproduction = 2,
    Alpha = 3,
    Beta = 4,

    Reserved5 = 0x5,
    Reserved6 = 0x6,
    Reserved7 = 0x7,
    Reserved8 = 0x8,
    Reserved9 = 0x9,
    ReservedA = 0xa,
    ReservedB = 0xb,
    ReservedC = 0xc,
    ReservedD = 0xd,

    ReservedE = 0xe,
    Production = 0xf,
}

#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, Debug, FromPrimitive, IntoPrimitive)]
#[repr(u8)]
pub enum ProductId {
    /// Canandmag
    Encoder = 0x1,
    /// Canandgyro
    Gyro = 0x2,
    /// Canandapter
    CanAdapter = 0x3,
    /// Canandcolor (don't question it)
    Sandworm = 0x4,
    /// Secret thing
    Neon = 0x5,
    /// Unreleased thing
    Nitrogen = 0x6,
    /// Other unreleased thing
    Nitro775 = 0x7,
    /// Zinc series
    Buck = 0x8,
    /// Third unreleased thing
    Nitrate = 0x9,
    #[num_enum(catch_all)]
    Unknown(u8),
}

/// Redux product serial number.
///
///
/// Serial numbers have:
///
/// * **Product ID** (8 bits; see above)
/// * **Revision ID** (4 bits)
/// * **Batch ID** (16 bits)
/// * **Device ID** (12 bits)
/// * **Lifecycle Flags** (4 bits; mule/proto/preprod/alpha/beta)
/// * **CRC** (4 bits)
///
/// -----------
///
/// The sane way to read the fields is right-to-left lexicographically.
///
/// For example, `SerialNumer([0xf4, 0x00, 0x20, 0x00, 0x02, 0x01])` would decode into:
///
/// * Product ID: 0x1
/// * Revision ID: 0x2
/// * Batch ID: 0x0000
/// * Device ID: 0x002
/// * Lifecycle Flag: 0x4
/// * CRC: 0xf
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct SerialNumer([u8; 6]);

impl SerialNumer {
    /// Generates a new serial numer from
    pub const fn new(s: [u8; 6]) -> Self {
        Self(s)
    }

    /// Generates a new serial numer from the fields, ensuring a valid CRC.
    pub fn build(
        product_id: ProductId,
        revision_id: u8,
        batch_id: u16,
        device_id: u16,
        lifecycle_flag: LifecycleFlag,
    ) -> Self {
        let mut s = [0u8; 6];
        s[0] = lifecycle_flag as u8;
        s[1] = (device_id >> 4) as u8;
        s[2] = ((device_id as u8) << 4) | (batch_id >> 12) as u8;
        s[3] = (batch_id >> 4) as u8;
        s[4] = ((batch_id << 4) as u8) | revision_id;
        s[5] = product_id.into();

        s[0] |= crc4itu_nibble_reverse(0, &s).1 << 4;

        Self::new(s)
    }

    pub fn is_zero(&self) -> bool {
        u32::from_le_bytes(self.0[0..4].try_into().unwrap()) == 0
            && u16::from_le_bytes(self.0[4..6].try_into().unwrap()) == 0
    }

    pub fn is_unset(&self) -> bool {
        u32::from_le_bytes(self.0[0..4].try_into().unwrap()) == 0xffffffff
            && u16::from_le_bytes(self.0[4..6].try_into().unwrap()) == 0xffff
    }

    /// Product id.
    pub fn product_id(&self) -> ProductId {
        ProductId::from(self.0[5])
    }

    // 2
    pub const fn revision_id(&self) -> u8 {
        self.0[4] & 0xf
    }

    pub const fn batch_id(&self) -> u16 {
        // 0 00 0
        ((self.0[2] as u16 & 0xf) << 12) | ((self.0[3] as u16) << 4) | (self.0[4] >> 4) as u16
    }

    pub const fn device_id(&self) -> u16 {
        // 00 | 2
        ((self.0[1] as u16) << 4) | (self.0[2] >> 4) as u16
    }

    pub fn lifecycle_flag(&self) -> LifecycleFlag {
        // 4
        LifecycleFlag::try_from(self.0[0] & 0xf).unwrap()
    }

    pub const fn crc(&self) -> u8 {
        // f
        self.0[0] >> 4
    }

    pub fn check_crc(&self) -> bool {
        crc4itu_nibble_reverse(0, &self.0).0 == 0
    }

    /// Converts the [`SerialNumer`] into a padded 64-bit array, suitable for transmission over CAN for things like
    /// ID arbitration packets.
    pub fn into_msg_padded(&self) -> [u8; 8] {
        let mut s = [0u8; 8];
        s[..6].copy_from_slice(&self.0);
        s
    }

    // convert a nibble to binary coded hex
    // plugging in anything larger than 0xf is Undefined Behavior
    fn to_bcx(a: u8) -> u8 {
        if a < 10 {
            a + b'0'
        } else {
            const DIFF: u8 = b'A' - 0xa;
            unsafe { a.unchecked_add(DIFF) }
        }
    }

    pub fn to_hex_str<'a>(&self, out_buf: &'a mut [u8; 12]) -> &'a str {
        for i in 0..5usize {
            unsafe {
                let v = *self.0.get_unchecked(i);
                out_buf[i << 1] = Self::to_bcx(v >> 4);
                out_buf[(i << 1) + 1] = Self::to_bcx(v & 0xf);
            }
        }
        unsafe { core::str::from_utf8_unchecked(out_buf) }
    }

    /// Creates serial numers of the form
    ///
    /// PP-R-BBBB-DDD-L-C
    /// |  | |    |   | +-- crc
    /// |  | |    |   +---- lifecycle flag
    /// |  | |    +-------- device id
    /// |  | +------------- batch id
    /// |  +--------------- revision id
    /// +------------------ product id
    ///
    /// in human-readable-endian hexadecimal, backed by out_buf.
    ///
    /// This is used in firmware for USB enumeration (hence the use of lifetimes),
    /// but also in software for human readability.
    pub fn to_readable_str<'a>(&self, out_buf: &'a mut [u8; 17]) -> &'a str {
        let product_id = self.0[5];
        out_buf[0] = Self::to_bcx(product_id >> 4);
        out_buf[1] = Self::to_bcx(product_id & 0xf);
        out_buf[2] = b'-';
        out_buf[3] = Self::to_bcx(self.revision_id());
        out_buf[4] = b'-';
        let batch_id = self.batch_id();
        out_buf[5] = Self::to_bcx(((batch_id >> 12) & 0xf) as u8);
        out_buf[6] = Self::to_bcx(((batch_id >> 8) & 0xf) as u8);
        out_buf[7] = Self::to_bcx(((batch_id >> 4) & 0xf) as u8);
        out_buf[8] = Self::to_bcx((batch_id & 0xf) as u8);
        out_buf[9] = b'-';
        let device_id = self.device_id();
        out_buf[10] = Self::to_bcx(((device_id >> 8) & 0xf) as u8);
        out_buf[11] = Self::to_bcx(((device_id >> 4) & 0xf) as u8);
        out_buf[12] = Self::to_bcx(((device_id) & 0xf) as u8);
        out_buf[13] = b'-';
        out_buf[14] = Self::to_bcx(self.0[0] & 0xf); // lifecycle flag
        out_buf[15] = b'-';
        out_buf[16] = Self::to_bcx(self.0[0] >> 4); // crc

        unsafe { core::str::from_utf8_unchecked(out_buf) }
    }

    /// Converts ASCII binary-coded-hexadecimal to a value.
    ///
    /// E.g. `b'3' -> 3` and `b'A' -> 0x10`.
    ///
    /// Returns [`None`] if not in correct range.
    fn from_bcx(a: u8) -> Option<u8> {
        let a_lower = a & 0b1011111;
        if a >= b'0' && a <= b'9' {
            Some(a - b'0')
        } else if a_lower >= b'A' && a_lower <= b'F' {
            Some(a_lower - b'A' + 10u8)
        } else {
            None
        }
    }

    /// Decodes a serial numer from the "readable" form, optionally allowing invalid CRCs.
    ///
    /// This is used for production-line flashing.
    pub fn from_readable_str(s: &str, allow_invalid_crc: bool) -> Option<SerialNumer> {
        let buf = s.as_bytes();
        if buf.len() < 17 {
            return None;
        } // serial numers are guarenteed to be 17 bytes long.
        let serial = SerialNumer::build(
            ProductId::from((Self::from_bcx(buf[0])? << 4) | Self::from_bcx(buf[1])?),
            Self::from_bcx(buf[3])?,
            ((Self::from_bcx(buf[5])? as u16) << 12)
                | ((Self::from_bcx(buf[6])? as u16) << 8)
                | ((Self::from_bcx(buf[7])? as u16) << 4)
                | (Self::from_bcx(buf[8])? as u16),
            ((Self::from_bcx(buf[10])? as u16) << 8)
                | ((Self::from_bcx(buf[11])? as u16) << 4)
                | (Self::from_bcx(buf[12])? as u16),
            LifecycleFlag::try_from(Self::from_bcx(buf[14])?).unwrap(),
        );
        let crc = serial.crc();
        if crc != Self::from_bcx(buf[16])? && !allow_invalid_crc {
            None
        } else {
            Some(serial)
        }
    }
}

impl AsRef<[u8; 6]> for SerialNumer {
    fn as_ref(&self) -> &[u8; 6] {
        &self.0
    }
}

impl From<[u8; 8]> for SerialNumer {
    fn from(value: [u8; 8]) -> Self {
        SerialNumer(value[..6].try_into().unwrap())
    }
}

impl From<u64> for SerialNumer {
    fn from(value: u64) -> Self {
        SerialNumer(value.to_le_bytes()[..6].try_into().unwrap())
    }
}

impl From<SerialNumer> for [u8; 6] {
    fn from(value: SerialNumer) -> Self {
        value.0
    }
}

impl From<SerialNumer> for [u8; 8] {
    fn from(value: SerialNumer) -> Self {
        value.into_msg_padded()
    }
}
