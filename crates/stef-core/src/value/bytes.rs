use std::io::Read;
use crate::error::StefCoreError;
use crate::value::bitsize::BitSize;
use crate::value::{utils, ReadOptions, StefValue};
use crate::value::typeid::TypeID;
use crate::types::{TypeByte, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Bytes {
    inner: Option<Vec<u8>>,
    nullable: bool,
}

impl Bytes {
    pub fn new_nullable(bytes: Vec<u8>, nullable: bool) -> Self {
        Self { inner: Some(bytes), nullable }
    }
    pub fn new(bytes: Vec<u8>) -> Self {
        Self::new_nullable(bytes, false)
    }
    pub fn null() -> Self {
        Self { inner: None, nullable: true }
    }
}

impl StefValue for Bytes {
    type Error = StefCoreError;

    fn type_id(&self) -> TypeID {
        TypeID::Bytes
    }

    fn bit_size(&self) -> Option<BitSize> {
        utils::bit_size_from_byte_sequence(self.inner.as_ref())
    }

    fn is_null(&self) -> bool {
        self.inner.is_none()
    }

    fn is_nullable(&self) -> bool {
        self.nullable
    }

    fn nullable(self) -> Result<Self> {
        Ok(Self {
            nullable: true,
            ..self
        })
    }

    fn non_nullable(self) -> Result<Self> {
        Ok(Self {
            nullable: false,
            ..self
        })
    }

    fn type_byte(&self) -> TypeByte {
        utils::type_byte_from_byte_sequence(self, self.nullable)
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        Ok(utils::serialize_byte_sequence(&self.inner, self.type_byte(), self.nullable))
    }

    fn deserialize(bytes: &mut impl Read, _depth: usize, _options: Option<&ReadOptions>) -> Result<Self> {
        let (inner, nullable) = utils::deserialize_byte_sequence(bytes, TypeID::Bytes)?;

        Ok(Self { inner, nullable })
    }
}

impl From<Vec<u8>> for Bytes {
    fn from(v: Vec<u8>) -> Self {
        Self::new(v)
    }
}

impl From<&[u8]> for Bytes {
    fn from(v: &[u8]) -> Self {
        Self::new(v.to_vec())
    }
}

impl From<Option<Vec<u8>>> for Bytes {
    fn from(v: Option<Vec<u8>>) -> Self {
        match v {
            Some(v) => Self::new(v),
            None => Self::null(),
        }
    }
}

impl TryFrom<Bytes> for Vec<u8> {
    type Error = StefCoreError;
    fn try_from(b: Bytes) -> Result<Self> {
        b.inner.ok_or(StefCoreError::IsNull)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- constructors ---

    #[test]
    fn new_creates_non_nullable() {
        let b = Bytes::new(vec![0x01, 0x02]);
        assert!(!b.is_nullable());
        assert!(!b.is_null());
    }

    #[test]
    fn null_creates_nullable_null() {
        let b = Bytes::null();
        assert!(b.is_nullable());
        assert!(b.is_null());
    }

    // --- type_byte ---

    #[test]
    fn type_byte_non_nullable_mini() {
        let b = Bytes::new(vec![0x01]);
        assert_eq!(b.type_byte(), 0b0_00_01001);
    }

    #[test]
    fn type_byte_nullable() {
        let b = Bytes::new(vec![0x01]).nullable().unwrap();
        assert_eq!(b.type_byte(), 0b1_00_01001);
    }

    // --- serialize ---

    #[test]
    fn serialize_non_nullable() {
        let b = Bytes::new(vec![0xDE, 0xAD, 0xBE, 0xEF]);
        assert_eq!(b.serialize().unwrap(), vec![
            0b0_00_01001, // type byte: non-nullable, Mini, Bytes
            0x04,         // length = 4
            0xDE, 0xAD, 0xBE, 0xEF
        ]);
    }

    #[test]
    fn serialize_nullable_present() {
        let b = Bytes::new(vec![0x01, 0x02]).nullable().unwrap();
        assert_eq!(b.serialize().unwrap(), vec![
            0b1_00_01001, // type byte: nullable, Mini, Bytes
            0x01,         // presence = present
            0x02,         // length = 2
            0x01, 0x02
        ]);
    }

    #[test]
    fn serialize_null() {
        let b = Bytes::null();
        assert_eq!(b.serialize().unwrap(), vec![
            0b1_00_01001, // type byte: nullable, Mini, Bytes
            0x00,         // presence = null
        ]);
    }

    // --- deserialize ---

    #[test]
    fn deserialize_non_nullable() {
        let bytes = vec![0b0_00_01001, 0x02, 0xAB, 0xCD];
        let b = Bytes::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert_eq!(Vec::<u8>::try_from(b).unwrap(), vec![0xAB, 0xCD]);
    }

    #[test]
    fn deserialize_nullable_present() {
        let bytes = vec![0b1_00_01001, 0x01, 0x02, 0xAB, 0xCD];
        let b = Bytes::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(b.is_nullable());
        assert_eq!(Vec::<u8>::try_from(b).unwrap(), vec![0xAB, 0xCD]);
    }

    #[test]
    fn deserialize_null() {
        let bytes = vec![0b1_00_01001, 0x00];
        let b = Bytes::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(b.is_null());
        assert!(matches!(Vec::<u8>::try_from(b), Err(StefCoreError::IsNull)));
    }

    #[test]
    fn deserialize_wrong_type_id() {
        let bytes = vec![0b0_00_00001, 0x01]; // uint, no bytes
        assert!(matches!(
            Bytes::deserialize(&mut bytes.as_slice(), 0, None),
            Err(StefCoreError::CannotReadAs(_))
        ));
    }

    // --- From/TryFrom ---

    #[test]
    fn from_vec() {
        let b = Bytes::from(vec![0x01, 0x02]);
        assert!(!b.is_null());
    }

    #[test]
    fn from_slice() {
        let b = Bytes::from([0x01, 0x02].as_slice());
        assert!(!b.is_null());
    }

    #[test]
    fn try_from_null_returns_error() {
        let b = Bytes::null();
        assert!(matches!(Vec::<u8>::try_from(b), Err(StefCoreError::IsNull)));
    }

    #[test]
    fn roundtrip() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        let b = Bytes::new(data.clone());
        let serialized = b.serialize().unwrap();
        let deserialized = Bytes::deserialize(&mut serialized.as_slice(), 0, None).unwrap();
        assert_eq!(Vec::<u8>::try_from(deserialized).unwrap(), data);
    }
}