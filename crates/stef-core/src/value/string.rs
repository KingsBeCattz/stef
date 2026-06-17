use std::io::Read;
use crate::error::StefCoreError;
use crate::value::bitsize::BitSize;
use crate::value::{ReadOptions, StefValue};
use crate::value::typeid::TypeID;
use crate::value::utils;
use crate::types::{TypeByte, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Text {
    /// UTF-8 encoded bytes
    inner: Option<Vec<u8>>,
    nullable: bool,
}

impl Text {
    pub fn new_nullable<T: ToString>(text: T, nullable: bool) -> Self {
        Self { inner: Some(text.to_string().as_bytes().to_vec()), nullable }
    }

    pub fn new<T: ToString>(text: T) -> Self {
        Self::new_nullable(text, false)
    }

    pub fn null() -> Self {
        Self { inner: None, nullable: true }
    }
}

impl StefValue for Text {
    type Error = StefCoreError;

    fn type_id(&self) -> TypeID {
        TypeID::String
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
        let (inner, nullable) = utils::deserialize_byte_sequence(bytes, TypeID::String)?;
        if let Some(inner) = &inner {
            String::from_utf8(inner.clone()).map_err(|_| StefCoreError::InvalidUtf8)?;
        };
        
        Ok(Self { inner, nullable })
    }
}

impl TryFrom<Text> for String {
    type Error = StefCoreError;
    fn try_from(t: Text) -> Result<Self> {
        if let Some(inner) = t.inner {
            String::from_utf8(inner)
                .map_err(|_| StefCoreError::InvalidUtf8)
        } else {
            Err(StefCoreError::IsNull)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- constructors ---

    #[test]
    fn new_creates_non_nullable() {
        let t = Text::new("hello");
        assert!(!t.is_nullable());
        assert!(!t.is_null());
    }

    #[test]
    fn null_creates_nullable_null() {
        let t = Text::null();
        assert!(t.is_nullable());
        assert!(t.is_null());
    }

    // --- type_byte ---

    #[test]
    fn type_byte_short_string_non_nullable() {
        let t = Text::new("hi"); // len=2, Mini
        assert_eq!(t.type_byte(), 0b0_00_01000);
    }

    #[test]
    fn type_byte_nullable() {
        let t = Text::new("hi").nullable().unwrap();
        assert_eq!(t.type_byte(), 0b1_00_01000);
    }

    // --- serialize ---

    #[test]
    fn serialize_non_nullable() {
        let t = Text::new("stef");
        // type_byte | length u8 | utf8 bytes
        assert_eq!(t.serialize().unwrap(), vec![
            0b0_00_01000, // type byte: non-nullable, Mini, String
            0x04,         // length = 4
            0x73, 0x74, 0x65, 0x66 // "stef"
        ]);
    }

    #[test]
    fn serialize_nullable_present() {
        let t = Text::new("hi").nullable().unwrap();
        assert_eq!(t.serialize().unwrap(), vec![
            0b1_00_01000, // type byte: nullable, Mini, String
            0x01,         // presence = present
            0x02,         // length = 2
            0x68, 0x69    // "hi"
        ]);
    }

    #[test]
    fn serialize_null() {
        let t = Text::null();
        assert_eq!(t.serialize().unwrap(), vec![
            0b1_00_01000, // type byte: nullable, Mini, String
            0x00,         // presence = null
        ]);
    }

    // --- deserialize ---

    #[test]
    fn deserialize_non_nullable() {
        let bytes = vec![0b0_00_01000, 0x04, 0x73, 0x74, 0x65, 0x66];
        let t = Text::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert_eq!(String::try_from(t).unwrap(), "stef");
    }

    #[test]
    fn deserialize_nullable_present() {
        let bytes = vec![0b1_00_01000, 0x01, 0x02, 0x68, 0x69];
        let t = Text::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(t.is_nullable());
        assert_eq!(String::try_from(t).unwrap(), "hi");
    }

    #[test]
    fn deserialize_null() {
        let bytes = vec![0b1_00_01000, 0x00];
        let t = Text::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(t.is_null());
        assert!(matches!(String::try_from(t), Err(StefCoreError::IsNull)));
    }

    #[test]
    fn deserialize_wrong_type_id() {
        let bytes = vec![0b0_00_00001, 0x01]; // uint, no string
        assert!(matches!(
            Text::deserialize(&mut bytes.as_slice(), 0, None),
            Err(StefCoreError::CannotReadAs(_))
        ));
    }

    // --- TryFrom ---

    #[test]
    fn try_from_null_returns_error() {
        let t = Text::null();
        assert!(matches!(String::try_from(t), Err(StefCoreError::IsNull)));
    }

    #[test]
    fn roundtrip() {
        let original = "hello stef";
        let t = Text::new(original);
        let serialized = t.serialize().unwrap();
        let deserialized = Text::deserialize(&mut serialized.as_slice(), 0, None).unwrap();
        assert_eq!(String::try_from(deserialized).unwrap(), original);
    }
}