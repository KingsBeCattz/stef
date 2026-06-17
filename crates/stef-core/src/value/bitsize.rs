#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum BitSize {
    /// 8 bits
    Mini = 0b0_00_00000,
    /// 16 bits
    Half = 0b0_01_00000,
    /// 32 bits
    Single = 0b0_10_00000,
    /// 64 bits
    Double = 0b0_11_00000,
}

impl From<BitSize> for u8 {
    fn from(b: BitSize) -> Self {
        b as u8
    }
}

impl BitSize {
    pub const fn mask() -> u8 {
        0b0_11_00000
    }
    pub fn as_bits(&self) -> u8 {
        match self {
            BitSize::Mini => 8,
            BitSize::Half => 16,
            BitSize::Single => 32,
            BitSize::Double => 64,
        }
    }
    pub fn as_bytes(&self) -> u8 {
        self.as_bits() / 8
    }
    pub fn from_bits(bits: u8) -> Self {
        match (bits & Self::mask()) >> 5 {
            0b00 => Self::Mini,
            0b01 => Self::Half,
            0b10 => Self::Single,
            0b11 => Self::Double,
            _    => unreachable!("How did you get here?"),
        }
    }
    pub fn find_size(value: u64) -> Self {
        if value <= u8::MAX as u64 {
            Self::Mini
        } else if value <= u16::MAX as u64 {
            Self::Half
        } else if value <= u32::MAX as u64 {
            Self::Single
        } else if value <= u64::MAX {
            Self::Double
        } else {
            panic!("How did you get here?");
        }
    }
}

impl std::fmt::Display for BitSize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_bits())
    }
}