use crate::value::bitsize::BitSize;
use crate::value::typeid::TypeID;
use crate::types::TypeByte;

#[derive(Debug)]
pub enum StefCoreError {
    InvalidTypeIdString(String),
    TypeMismatch { expected: TypeID, found: TypeID },
    SizeOverflow { requested: BitSize, actual: BitSize },
    IsNull,
    Io(std::io::Error),
    FloatVariantReserved(BitSize),
    InvalidTypeId(u8),
    CannotReadAs(TypeID),
    NullableNotSupported(TypeID),
    InvalidUtf8,
    TypeByteMismatch { expected: TypeByte, found: TypeByte },
    ExceededMaxArraySize(isize),
    IndexOutOfBounds { index: usize, max: usize },
    MaximumDepthExceeded {
        depth: usize,
        max_depth: usize
    },
    NameTooLong {
        max: u8,
        actual: usize,
    },
    CannotDeserialize(TypeID),
    InvalidMagic,
    UnsupportedVersion(u8),
    ChecksumMismatch,
    UnexpectedTopLevelRecord(String),
}