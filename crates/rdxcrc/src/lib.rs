//! RdxCRC: Common CRC stuff for Redux products.
#![no_std]

/// Common trait for CRC32 impls
pub trait Crc32 {
    fn init(&mut self);
    fn update(&mut self, data: &[u32]) -> u32;
    fn update_bytes(&mut self, data: &[u8]) -> u32;
}

const TABLE_NIBBLE: [u8; 16] = [
    0x0, 0xD, 0x3, 0xE, 0x6, 0xB, 0x5, 0x8, 0xC, 0x1, 0xF, 0x2, 0xA, 0x7, 0x9, 0x4,
];

/// used in serial number validation
pub fn crc4itu_nibble_reverse(init: u8, data: &[u8]) -> (u8, u8) {
    if data.len() == 0 {
        return (0, 0);
    }
    let mut crc = init & 0xf;
    let mut lag = crc;

    for b in data.iter().rev() {
        // SAFETY: `TABLE_NIBBLE` is 16 elements long and the masking operations constrain the value to < 16.
        unsafe {
            crc = *TABLE_NIBBLE.get_unchecked((crc ^ (*b & 0xf)) as usize);
            lag = crc;
            crc = *TABLE_NIBBLE.get_unchecked((crc ^ (*b >> 4)) as usize);
        }
    }

    (crc, lag)
}

const CRC32_MPEG2_TABLE: [u32; 16] = [
    0x00000000, 0x04C11DB7, 0x09823B6E, 0x0D4326D9, 0x130476DC, 0x17C56B6B, 0x1A864DB2, 0x1E475005,
    0x2608EDB8, 0x22C9F00F, 0x2F8AD6D6, 0x2B4BCB61, 0x350C9B64, 0x31CD86D3, 0x3C8EA00A, 0x384FBDBD,
];

/// Software implementation of CRC32/mpeg2.
///
/// This code is adapted from [this StackOverflow answer.](https://stackoverflow.com/a/31602216)
///
/// Note that if the input is not 4-byte aligned many hardware implementations will compute assuming extra padding 0s.
/// this does not add the padding, so implementations must add that manually or guarentee 4-alignment to ensure compatibility.
pub fn crc32_mpeg2(mut crc: u32, data: &[u8]) -> u32 {
    for b in data {
        crc = crc ^ ((*b as u32) << 24);
        // SAFETY: The table is 16 elements long and the masking operations constrain the value to < 16.
        unsafe {
            crc = (crc << 4) ^ CRC32_MPEG2_TABLE.get_unchecked((crc >> 28) as usize);
            crc = (crc << 4) ^ CRC32_MPEG2_TABLE.get_unchecked((crc >> 28) as usize);
        }
    }
    crc
}

/// Software implementation of crc32/mpeg2 that applies 4-byte padding for consistency with hardware implementations.
pub fn crc32_mpeg2_pad(crc: u32, data: &[u8]) -> u32 {
    let align = data.len() & 0b11;
    if align == 0 {
        crc32_mpeg2(crc, data)
    } else {
        const PAD: [u8; 4] = [0u8; 4];
        crc32_mpeg2(crc32_mpeg2(crc, data), &PAD[..4usize - align])
    }
}

/// Software CRC32 implementation
#[derive(Debug)]
pub struct SoftwareCrc32 {
    value: u32,
}

impl SoftwareCrc32 {
    pub fn new() -> Self {
        Self { value: 0xffff_ffff }
    }
}

impl Crc32 for SoftwareCrc32 {
    fn init(&mut self) {
        self.value = 0xffff_ffff;
    }

    fn update(&mut self, data: &[u32]) -> u32 {
        for word in data {
            self.value = crc32_mpeg2(self.value, &word.to_le_bytes());
        }
        self.value
    }

    fn update_bytes(&mut self, data: &[u8]) -> u32 {
        self.value = crc32_mpeg2_pad(self.value, data);
        self.value
    }
}
