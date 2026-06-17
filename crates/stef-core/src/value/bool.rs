use std::io::Read;
use crate::error::StefCoreError;
use crate::readoptions::ReadOptions;
use crate::value::bitsize::BitSize;
use crate::value::StefValue;
use crate::value::typeid::TypeID;
use crate::types::{TypeByte, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bool {
    state: bool,
}

impl Bool {
    pub fn new(state: bool) -> Self {
        Bool { state }
    }
}

impl StefValue for Bool {
    type Error = StefCoreError;

    fn type_id(&self) -> TypeID {
        TypeID::Bool
    }

    fn bit_size(&self) -> Option<BitSize> {
        None
    }

    fn nullable(self) -> Result<Self> {
        Err(Self::Error::NullableNotSupported((&self).type_id()))
    }

    fn non_nullable(self) -> Result<Self> {
        Ok(self)
    }

    fn type_byte(&self) -> TypeByte {
        TypeID::Bool as u8
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(vec![self.type_byte(), if self.state { 0x01 } else { 0x00 }])
    }

    fn deserialize(bytes: &mut impl Read, _depth: usize, _options: Option<&ReadOptions>) -> Result<Self> {
        let mut type_byte = [0u8; 1];
        bytes.read_exact(&mut type_byte)
            .map_err(StefCoreError::Io)?;

        let type_id = TypeID::from_bits(type_byte[0])
            .ok_or(Self::Error::InvalidTypeId(type_byte[0]))?;

        if type_id != TypeID::Bool {
            return Err(Self::Error::TypeMismatch {
                expected: TypeID::Bool,
                found: type_id,
            });
        }

        let mut value = [0u8; 1];
        bytes.read_exact(&mut value)
            .map_err(StefCoreError::Io)?;

        Ok(Bool { state: value[0] != 0x00 })
    }
}

impl From<Bool> for bool {
    fn from(b: Bool) -> Self {
        b.state
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- type_byte ---

    #[test]
    fn type_byte_is_correct() {
        let b = Bool::new(true);
        assert_eq!(b.type_byte(), 0b0_00_00100);
    }

    // --- serialize ---

    #[test]
    fn serialize_true() {
        let b = Bool::new(true);
        assert_eq!(b.serialize().unwrap(), vec![0b0_00_00100, 0x01]);
    }

    #[test]
    fn serialize_false() {
        let b = Bool::new(false);
        assert_eq!(b.serialize().unwrap(), vec![0b0_00_00100, 0x00]);
    }

    // --- deserialize ---

    #[test]
    fn deserialize_true() {
        let bytes = vec![0b0_00_00100, 0x01];
        let b = Bool::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(b.state);
    }

    #[test]
    fn deserialize_false() {
        let bytes = vec![0b0_00_00100, 0x00];
        let b = Bool::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(!b.state);
    }

    #[test]
    fn deserialize_nonzero_is_true() {
        let bytes = vec![0b0_00_00100, 0xFF];
        let b = Bool::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(b.state);
    }

    #[test]
    fn deserialize_ignores_nullable_flag() {
        // Nullable flag en type byte debe ser ignorado
        let bytes = vec![0b1_00_00100, 0x01];
        let b = Bool::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(b.state);
    }

    #[test]
    fn deserialize_wrong_type_id() {
        let bytes = vec![0b0_00_00001, 0x01]; // uint, no bool
        assert!(matches!(
            Bool::deserialize(&mut bytes.as_slice(), 0, None),
            Err(StefCoreError::TypeMismatch { .. })
        ));
    }

    // --- nullable ---

    #[test]
    fn nullable_returns_error() {
        let b = Bool::new(true);
        assert!(matches!(
            b.nullable(),
            Err(StefCoreError::NullableNotSupported(TypeID::Bool))
        ));
    }

    #[test]
    fn non_nullable_is_noop() {
        let b = Bool::new(true);
        assert!(b.non_nullable().is_ok());
    }

    // --- is_nullable / is_null ---

    #[test]
    fn is_never_nullable() {
        let b = Bool::new(true);
        assert!(!b.is_nullable());
        assert!(!b.is_null());
    }
}