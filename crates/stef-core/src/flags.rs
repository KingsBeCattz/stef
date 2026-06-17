use crate::constants::flags as constants;


#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Flags {
    pub checksum: bool,
    pub compressed: bool,
}

impl Flags {
    pub fn set_checksum(&mut self, checksum: bool) {
        self.checksum = checksum;
    }
    pub fn set_compressed(&mut self, compressed: bool) {
        self.compressed = compressed;
    }
    pub fn is_checksum(&self) -> bool {
        self.checksum
    }
    pub fn is_compressed(&self) -> bool {
        self.compressed
    }
}

impl Default for Flags {
    fn default() -> Self {
        Flags {
            checksum: false,
            compressed: false,
        }
    }
}

impl From<&Flags> for u8 {
    fn from(flags: &Flags) -> Self {
        let mut flags_byte = 0u8;
        if flags.checksum {
            flags_byte |= constants::CHECKSUM_FLAG;
        }
        if flags.compressed {
            flags_byte |= constants::COMPRESSED_FLAG;
        }
        flags_byte
    }
}

impl From<u8> for Flags {
    fn from(flags: u8) -> Self {
        Flags {
            checksum: (flags & constants::CHECKSUM_FLAG) != 0,
            compressed: (flags & constants::COMPRESSED_FLAG) != 0,
        }
    }
}