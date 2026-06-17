use crate::error::StefCoreError;

pub mod as_string {
    pub const UINT: &str = "uint";
    pub const INT: &str = "int";
    pub const FLOAT: &str = "float";
    pub const BOOL: &str = "bool";
    pub const STRING: &str = "string";
    pub const BYTES: &str = "bytes";
    pub const HETERO_ARRAY: &str = "hetero_array";
    pub const HOMO_ARRAY: &str = "homo_array";
    pub const RECORD: &str = "record";
    pub const CHECKSUM: &str = "checksum";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum TypeID {
    Uint = 0b000_00001,
    Int = 0b000_00010,
    Float = 0b000_00011,
    Bool = 0b000_00100,

    String = 0b000_01000,
    Bytes = 0b000_01001,

    HeteroArray = 0b000_10000,
    HomoArray = 0b000_10001,
    Record = 0b000_10010
}

impl From<TypeID> for u8 {
    fn from(b: TypeID) -> Self {
        b as u8
    }
}

impl TypeID {
    pub const fn mask() -> u8 {
        0b000_11111
    }
    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits & Self::mask() {
            0b000_00001 => Some(TypeID::Uint),
            0b000_00010 => Some(TypeID::Int),
            0b000_00011 => Some(TypeID::Float),
            0b000_00100 => Some(TypeID::Bool),
            0b000_01000 => Some(TypeID::String),
            0b000_01001 => Some(TypeID::Bytes),
            0b000_10000 => Some(TypeID::HeteroArray),
            0b000_10001 => Some(TypeID::HomoArray),
            0b000_10010 => Some(TypeID::Record),
            _ => None
        }
    }
}

impl std::fmt::Display for TypeID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            TypeID::Uint => as_string::UINT,
            TypeID::Int => as_string::INT,
            TypeID::Float => as_string::FLOAT,
            TypeID::Bool => as_string::BOOL,
            TypeID::String => as_string::STRING,
            TypeID::Bytes => as_string::BYTES,
            TypeID::HeteroArray => as_string::HETERO_ARRAY,
            TypeID::HomoArray => as_string::HOMO_ARRAY,
            TypeID::Record => as_string::RECORD,
        })
    }
}

impl std::str::FromStr for TypeID {
    type Err = StefCoreError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            as_string::UINT => Ok(TypeID::Uint),
            as_string::INT => Ok(TypeID::Int),
            as_string::FLOAT => Ok(TypeID::Float),
            as_string::BOOL => Ok(TypeID::Bool),
            as_string::STRING => Ok(TypeID::String),
            as_string::BYTES => Ok(TypeID::Bytes),
            as_string::HETERO_ARRAY => Ok(TypeID::HeteroArray),
            as_string::HOMO_ARRAY => Ok(TypeID::HomoArray),
            as_string::RECORD => Ok(TypeID::Record),
            _ => Err(Self::Err::InvalidTypeIdString(s.into()))
        }
    }
}