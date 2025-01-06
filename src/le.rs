#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U64(u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct U8(u8);

impl U64 {
    pub fn new(i: u64) -> Self {
        if cfg!(target_endian = "little") {
            Self(i)
        } else {
            Self(u64::from_le_bytes(u64::to_le_bytes(i)))
        }
    }

    pub fn get(self) -> u64 {
        self.0
    }
}

impl U8 {
    pub fn new(i: u8) -> Self {
        if cfg!(target_endian = "little") {
            Self(i)
        } else {
            Self(u8::from_le_bytes(u8::to_le_bytes(i)))
        }
    }

    pub fn get(self) -> u8 {
        self.0
    }
}
