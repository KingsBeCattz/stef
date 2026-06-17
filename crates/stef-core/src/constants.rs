/// Constant to validate the file format
pub const MAGIC: &'static [u8;4] = b"STEF";
/// Current version of the core to validate the file format
pub const VERSION: u8 = 0x01;
/// Name of the root record
pub const ROOT_NAME: &'static str = "root";
/// Name of the meta-record
pub const META_NAME: &'static str = "meta";
/// Constants for flags
pub mod flags {
    pub const CHECKSUM_FLAG: u8 = 1 << 0;
    pub const COMPRESSED_FLAG: u8 = 1 << 1;
}
/// Constants for writing
pub mod writing {
    pub const MAX_NAME_LENGTH: usize = 255;
}
/// Constants for reading
pub mod reading {
    /// Mask to check if a type is nullable
    pub const NULLABLE_MASK: u8 = 0b1_00_00000;
    /// Default maximum depth of the deserialization on a record and array
    pub const DEFAULT_MAX_DEPTH: usize = 64;
    /// Default max bytes for strings and raw bytes
    pub const DEFAULT_MAX_BYTES: u64 = 16 * 1024 * 1024;
    /// Default max elements for arrays
    pub const DEFAULT_MAX_ELEMENTS: u64 = 1_000_000;
    /// Default max fields for records
    pub const DEFAULT_MAX_FIELDS: u64 = 10_000;
}
