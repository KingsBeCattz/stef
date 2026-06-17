use std::io::Read;
use crate::constants;
use crate::error::StefCoreError;
use crate::value::bitsize::BitSize;
use crate::value::typeid::TypeID;
use crate::value::StefValue;
use crate::types::TypeByte;

pub(crate) fn write_length(bytes: &mut Vec<u8>, length: u64, size: BitSize) {
    match size {
        BitSize::Mini => bytes.extend_from_slice(&(length as u8).to_be_bytes()),
        BitSize::Half => bytes.extend_from_slice(&(length as u16).to_be_bytes()),
        BitSize::Single => bytes.extend_from_slice(&(length as u32).to_be_bytes()),
        BitSize::Double => bytes.extend_from_slice(&length.to_be_bytes()),
    }
}

pub(crate) fn read_length(
    bytes: &mut impl Read,
    size: BitSize,
) -> Result<u64, std::io::Error> {
    match size {
        BitSize::Mini => {
            let mut length = [0u8; 1];
            bytes.read_exact(&mut length)?;
            Ok(u8::from_be_bytes(length) as u64)
        }
        BitSize::Half => {
            let mut length = [0u8; 2];
            bytes.read_exact(&mut length)?;
            Ok(u16::from_be_bytes(length) as u64)
        }
        BitSize::Single => {
            let mut length = [0u8; 4];
            bytes.read_exact(&mut length)?;
            Ok(u32::from_be_bytes(length) as u64)
        }
        BitSize::Double => {
            let mut length = [0u8; 8];
            bytes.read_exact(&mut length)?;
            Ok(u64::from_be_bytes(length))
        }
    }
}

pub(crate) fn type_byte_from_byte_sequence<V: StefValue>(value: &V, nullable: bool) -> TypeByte {
    let null_byte: u8 = if nullable { constants::reading::NULLABLE_MASK } else { 0 };
    null_byte | value.bit_size().unwrap_or(BitSize::Mini) as u8 | value.type_id() as u8
}

pub(crate) fn bit_size_from_byte_sequence<T>(inner: Option<&Vec<T>>) -> Option<BitSize> {
    if let Some(inner) = inner {
        Some(BitSize::find_size(inner.len() as u64))
    } else {
        None
    }
}

pub(crate) fn serialize_byte_sequence(
    inner: &Option<Vec<u8>>,
    type_byte: TypeByte,
    nullable: bool,
) -> Vec<u8> {
    let mut bytes = vec![type_byte];
    if nullable {
        bytes.push(if inner.is_some() { 0x01 } else { 0x00 });
    }
    if let Some(inner) = &inner {
        let size = BitSize::find_size(inner.len() as u64);
        write_length(&mut bytes, inner.len() as u64, size);
        bytes.extend_from_slice(inner);
    }
    bytes
}

pub(crate) fn deserialize_byte_sequence(bytes: &mut impl Read, target: TypeID) -> Result<(Option<Vec<u8>>, bool), StefCoreError> {
    let mut type_byte = [0u8; 1];
    bytes
        .read_exact(&mut type_byte)
        .map_err(StefCoreError::Io)?;
    let type_byte: TypeByte = type_byte[0];
    let nullable = (type_byte & constants::reading::NULLABLE_MASK) != 0;

    let type_id =
        TypeID::from_bits(type_byte).ok_or(StefCoreError::InvalidTypeId(type_byte))?;

    if type_id != target {
        return Err(StefCoreError::CannotReadAs(type_id));
    }

    let size = BitSize::from_bits(type_byte);

    if nullable {
        let mut presence = [0u8; 1];
        bytes.read_exact(&mut presence).map_err(StefCoreError::Io)?;

        if presence[0] == 0x00 {
            return Ok((None, true));
        }
    }

    let length = read_length(bytes, size).map_err(StefCoreError::Io)?;
    let mut inner = vec![0u8; length as usize];
    bytes.read_exact(&mut inner).map_err(StefCoreError::Io)?;

    Ok((Some(inner), nullable))
}