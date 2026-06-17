pub mod typeid;
pub mod bitsize;
pub mod bool;
pub mod bytes;
pub mod number;
pub mod string;
pub mod homoarray;
pub mod heteroarray;
pub mod record;
pub(crate) mod utils;

use std::io::Read;

use typeid::TypeID;
use bitsize::BitSize;
use bool::*;
use bytes::*;
use number::*;
use string::*;
use heteroarray::*;
use homoarray::*;
use record::*;

use crate::readoptions::ReadOptions;
use crate::types::{TypeByte, Result};

pub trait StefValue: Sized {
    type Error;
    fn type_id(&self) -> TypeID;
    fn bit_size(&self) -> Option<BitSize>;
    fn is_null(&self) -> bool {
        false
    }
    fn is_nullable(&self) -> bool {
        false
    }
    fn nullable(self) -> Result<Self>;
    fn non_nullable(self) -> Result<Self>;
    fn type_byte(&self) -> TypeByte;
    fn serialize(&self) -> Result<Vec<u8>>;
    fn deserialize(bytes: &mut impl Read, depth: usize, options: Option<&ReadOptions>) -> Result<Self>;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Value {
    Number(Number),
    Bool(Bool),
    Text(Text),
    Bytes(Bytes),
    HomoArray(HomoArray),
    HeteroArray(HeteroArray),
    Record(Record),
}

impl StefValue for Value {
    type Error = crate::error::StefCoreError;
    fn type_id(&self) -> TypeID {
        match self {
            Value::Number(n) => n.type_id(),
            Value::Bool(_) => TypeID::Bool,
            Value::Text(_) => TypeID::String,
            Value::Bytes(_) => TypeID::Bytes,
            Value::HomoArray(_) => TypeID::HomoArray,
            Value::HeteroArray(_) => TypeID::HeteroArray,
            Value::Record(_) => TypeID::Record,
        }
    }

    fn bit_size(&self) -> Option<BitSize> {
        match self {
            Value::Number(n) => n.bit_size(),
            Value::Bool(b) => b.bit_size(),
            Value::Text(t) => t.bit_size(),
            Value::Bytes(b) => b.bit_size(),
            Value::HomoArray(homo_array) => homo_array.bit_size(),
            Value::HeteroArray(hetero_array) => hetero_array.bit_size(),
            Value::Record(r) => r.bit_size(),
        }
    }

    fn is_null(&self) -> bool {
        match self {
            Value::Number(n) => n.is_null(),
            Value::Bool(b) => b.is_null(),
            Value::Text(t) => t.is_null(),
            Value::Bytes(b) => b.is_null(),
            Value::HomoArray(homo_array) => homo_array.is_null(),
            Value::HeteroArray(hetero_array) => hetero_array.is_null(),
            Value::Record(r) => r.is_null(),
        }
    }

    fn is_nullable(&self) -> bool {
        match self {
            Value::Number(n) => n.is_nullable(),
            Value::Bool(b) => b.is_nullable(),
            Value::Text(t) => t.is_nullable(),
            Value::Bytes(b) => b.is_nullable(),
            Value::HomoArray(homo_array) => homo_array.is_nullable(),
            Value::HeteroArray(hetero_array) => hetero_array.is_nullable(),
            Value::Record(r) => r.is_nullable(),
        }
    }

    fn nullable(self) -> Result<Self> {
        match self {
            Value::Number(n) => Ok(Value::Number(n.nullable()?)),
            Value::Bool(b) => Ok(Value::Bool(b.nullable()?)),
            Value::Text(t) => Ok(Value::Text(t.nullable()?)),
            Value::Bytes(b) => Ok(Value::Bytes(b.nullable()?)),
            Value::HomoArray(homo_array) => Ok(Value::HomoArray(homo_array.nullable()?)),
            Value::HeteroArray(hetero_array) => Ok(Value::HeteroArray(hetero_array.nullable()?)),
            Value::Record(r) => Ok(Value::Record(r.nullable()?)),
        }
    }

    fn non_nullable(self) -> Result<Self> {
        match self {
            Value::Number(n) => Ok(Value::Number(n.non_nullable()?)),
            Value::Bool(b) => Ok(Value::Bool(b.non_nullable()?)),
            Value::Text(t) => Ok(Value::Text(t.non_nullable()?)),
            Value::Bytes(b) => Ok(Value::Bytes(b.non_nullable()?)),
            Value::HomoArray(homo_array) => Ok(Value::HomoArray(homo_array.non_nullable()?)),
            Value::HeteroArray(hetero_array) => Ok(Value::HeteroArray(hetero_array.non_nullable()?)),
            Value::Record(r) => Ok(Value::Record(r.non_nullable()?)),
        }
    }

    fn type_byte(&self) -> TypeByte {
        match self {
            Value::Number(n) => n.type_byte(),
            Value::Bool(b) => b.type_byte(),
            Value::Text(t) => t.type_byte(),
            Value::Bytes(b) => b.type_byte(),
            Value::HomoArray(homo_array) => homo_array.type_byte(),
            Value::HeteroArray(hetero_array) => hetero_array.type_byte(),
            Value::Record(r) => r.type_byte(),
        }
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        match self {
            Value::Number(n) => n.serialize(),
            Value::Bool(b) => b.serialize(),
            Value::Text(t) => t.serialize(),
            Value::Bytes(b) => b.serialize(),
            Value::HomoArray(homo_array) => homo_array.serialize(),
            Value::HeteroArray(hetero_array) => hetero_array.serialize(),
            Value::Record(r) => r.serialize(),
        }
    }

    fn deserialize(bytes: &mut impl Read, depth: usize, options: Option<&ReadOptions>) -> Result<Self> {
        let mut type_byte = [0u8; 1];
        bytes.read_exact(&mut type_byte).map_err(Self::Error::Io)?;

        let type_id = TypeID::from_bits(type_byte[0])
            .ok_or(Self::Error::InvalidTypeId(type_byte[0]))?;

        let full_bytes = type_byte.as_slice().chain(bytes as &mut dyn Read);
        let mut full_bytes = full_bytes;

        match type_id {
            TypeID::Uint | TypeID::Int | TypeID::Float => {
                Ok(Value::Number(Number::deserialize(&mut full_bytes, depth, options)?))
            }
            TypeID::Bool => {
                Ok(Value::Bool(Bool::deserialize(&mut full_bytes, depth, options)?))
            }
            TypeID::String => {
                Ok(Value::Text(Text::deserialize(&mut full_bytes, depth, options)?))
            }
            TypeID::Bytes => {
                Ok(Value::Bytes(Bytes::deserialize(&mut full_bytes, depth, options)?))
            }
            TypeID::HomoArray => {
                Ok(Value::HomoArray(HomoArray::deserialize(&mut full_bytes, depth, options)?))
            }
            TypeID::HeteroArray => {
                Ok(Value::HeteroArray(HeteroArray::deserialize(&mut full_bytes, depth, options)?))
            },
            TypeID::Record => {
                Ok(Value::Record(Record::deserialize(&mut full_bytes, depth, options)?))
            }
        }
    }
}

impl From<Number> for Value {
    fn from(number: Number) -> Self {
        Value::Number(number)
    }
}

impl From<Bool> for Value {
    fn from(bool: Bool) -> Self {
        Value::Bool(bool)
    }
}

impl From<Text> for Value {
    fn from(text: Text) -> Self {
        Value::Text(text)
    }
}

impl From<Bytes> for Value {
    fn from(bytes: Bytes) -> Self {
        Value::Bytes(bytes)
    }
}

impl From<HeteroArray> for Value {
    fn from(hetero_array: HeteroArray) -> Self {
        Value::HeteroArray(hetero_array)
    }
}

impl From<HomoArray> for Value {
    fn from(homo_array: HomoArray) -> Self {
        Value::HomoArray(homo_array)
    }
}

impl From<Record> for Value {
    fn from(record: Record) -> Self {
        Value::Record(record)
    }
}