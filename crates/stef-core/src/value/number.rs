use std::io::Read;
use crate::constants;
use crate::error::StefCoreError;
use crate::value::bitsize::BitSize;
use crate::value::{ReadOptions, StefValue};
use crate::value::typeid::TypeID;
use crate::types::{TypeByte, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Number {
    /// Big endian number bytes
    inner: Option<[u8; 8]>,
    size: BitSize,
    type_id: TypeID,
    nullable: bool,
}

impl Number {
    fn overflows(&self, size: BitSize) -> bool {
        self.size > size
    }
    fn overflow_error(&self, size: BitSize) -> StefCoreError {
        StefCoreError::SizeOverflow {
            requested: size,
            actual: self.size
        }
    }
    fn type_mismatch_error(&self, expected: TypeID) -> StefCoreError {
        StefCoreError::TypeMismatch {
            expected,
            found: self.type_id
        }
    }
    pub fn uint_nullable(value: u64, size: BitSize, nullable: bool) -> Self {
        let mut inner = [0u8; 8];
        let byte_count = size.as_bytes() as usize;
        inner[8 - byte_count..].copy_from_slice(&value.to_be_bytes()[8 - byte_count..]);
        Number { inner: Some(inner), size, type_id: TypeID::Uint, nullable }
    }

    pub fn uint(value: u64, size: BitSize) -> Self {
        Self::uint_nullable(value, size, false)
    }

    pub fn int_nullable(value: i64, size: BitSize, nullable: bool) -> Self {
        let mut inner = [0u8; 8];
        let byte_count = size.as_bytes() as usize;
        inner[8 - byte_count..].copy_from_slice(&value.to_be_bytes()[8 - byte_count..]);
        Number { inner: Some(inner), size, type_id: TypeID::Int, nullable }
    }

    pub fn int(value: i64, size: BitSize) -> Self {
        Self::int_nullable(value, size, false)
    }

    pub fn float_single_nullable(value: f32, nullable: bool) -> Self {
        let mut inner = [0u8; 8];
        inner[4..].copy_from_slice(&value.to_be_bytes());
        Number { inner: Some(inner), size: BitSize::Single, type_id: TypeID::Float, nullable }
    }

    pub fn float_single(value: f32) -> Self {
        Self::float_single_nullable(value, false)
    }

    pub fn float_double_nullable(value: f64, nullable: bool) -> Self {
        Number { inner: Some(value.to_be_bytes()), size: BitSize::Double, type_id: TypeID::Float, nullable }
    }

    pub fn float_double(value: f64) -> Self {
        Self::float_double_nullable(value, false)
    }

    pub fn null(size: BitSize, type_id: TypeID) -> Self {
        if !matches!(type_id, TypeID::Uint | TypeID::Int | TypeID::Float) {
            panic!("You cannot create a null number of type {:?}", type_id);
        }
        Number { inner: None, size, type_id, nullable: true }
    }
}

impl StefValue for Number {
    type Error = StefCoreError;
    fn type_id(&self) -> TypeID {
        self.type_id
    }

    fn bit_size(&self) -> Option<BitSize> {
        Some(self.size)
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
        let null_byte: u8 = if self.nullable { constants::reading::NULLABLE_MASK } else { 0 };
        null_byte | self.size as u8 | self.type_id as u8
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let type_byte = self.type_byte();
        let mut bytes = vec![type_byte];
        if self.nullable {
            bytes.push(if self.inner.is_some() { 0x01 } else { 0x00 });
        }
        if let Some(inner) = &self.inner {
            let byte_count = self.size.as_bytes() as usize;
            bytes.extend_from_slice(&inner[8 - byte_count..]);
        }
        Ok(bytes)
    }

    fn deserialize(bytes: &mut impl Read, _depth: usize, _options: Option<&ReadOptions>) -> Result<Self> {
        let mut type_byte = [0u8;1];
        bytes.read_exact(&mut type_byte)
            .map_err(StefCoreError::Io)?;
        let type_byte = type_byte[0];
        let nullable = (type_byte & constants::reading::NULLABLE_MASK) != 0;
        let type_id = TypeID::from_bits(type_byte)
            .ok_or(StefCoreError::InvalidTypeId(type_byte))?;

        if !matches!(type_id, TypeID::Uint | TypeID::Int | TypeID::Float) {
            return Err(StefCoreError::CannotReadAs(type_id));
        }

        let size = BitSize::from_bits(type_byte);

        if type_id == TypeID::Float && size < BitSize::Single {
            return Err(StefCoreError::FloatVariantReserved(size));
        }

        if nullable {
            let mut presence = [0u8; 1];
            bytes.read_exact(&mut presence)
                .map_err(StefCoreError::Io)?;

            if presence[0] == 0x00 {
                return Ok(Number { inner: None, size, type_id, nullable });
            }
        }

        let byte_count = size.as_bytes() as usize;
        let mut inner = [0u8; 8];
        bytes.read_exact(&mut inner[8 - byte_count..])
            .map_err(StefCoreError::Io)?;

        Ok(Number { inner: Some(inner), size, type_id, nullable })
    }
}

impl TryFrom<Number> for u8 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Uint {
            return Err(n.type_mismatch_error(TypeID::Uint));
        }
        if n.overflows(BitSize::Mini) {
            return Err(n.overflow_error(BitSize::Mini));
        }
        match n.inner {
            Some(inner) => Ok(inner[7]),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for i8 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Int {
            return Err(n.type_mismatch_error(TypeID::Int));
        }

        if n.overflows(BitSize::Mini) {
            return Err(n.overflow_error(BitSize::Mini));
        }

        match n.inner {
            Some(inner) => Ok(inner[7] as i8),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for u16 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Uint {
            return Err(n.type_mismatch_error(TypeID::Uint));
        }

        if n.overflows(BitSize::Half) {
            return Err(n.overflow_error(BitSize::Half));
        }

        match n.inner {
            Some(inner) => Ok(u16::from_be_bytes(inner[6..8].try_into().unwrap())),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for i16 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Int {
            return Err(n.type_mismatch_error(TypeID::Int));
        }

        if n.overflows(BitSize::Half) {
            return Err(n.overflow_error(BitSize::Half));
        }

        match n.inner {
            Some(inner) => Ok(i16::from_be_bytes(inner[6..8].try_into().unwrap())),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for u32 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Uint {
            return Err(n.type_mismatch_error(TypeID::Uint));
        }

        if n.overflows(BitSize::Single) {
            return Err(n.overflow_error(BitSize::Single));
        }

        match n.inner {
            Some(inner) => Ok(u32::from_be_bytes(inner[4..8].try_into().unwrap())),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for i32 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Int {
            return Err(n.type_mismatch_error(TypeID::Int));
        }

        if n.overflows(BitSize::Single) {
            return Err(n.overflow_error(BitSize::Single));
        }

        match n.inner {
            Some(inner) => Ok(i32::from_be_bytes(inner[4..8].try_into().unwrap())),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for u64 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Uint {
            return Err(n.type_mismatch_error(TypeID::Uint));
        }

        if n.overflows(BitSize::Double) {
            return Err(n.overflow_error(BitSize::Double));
        }

        match n.inner {
            Some(inner) => Ok(u64::from_be_bytes(inner)),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for i64 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Int {
            return Err(n.type_mismatch_error(TypeID::Int));
        }

        if n.overflows(BitSize::Double) {
            return Err(n.overflow_error(BitSize::Double));
        }

        match n.inner {
            Some(inner) => Ok(i64::from_be_bytes(inner)),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for f32 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Float {
            return Err(n.type_mismatch_error(TypeID::Float));
        }

        if n.overflows(BitSize::Single) {
            return Err(n.overflow_error(BitSize::Single));
        }

        match n.inner {
            Some(inner) => Ok(f32::from_be_bytes(inner[4..8].try_into().unwrap())),
            None => Err(StefCoreError::IsNull),
        }
    }
}

impl TryFrom<Number> for f64 {
    type Error = StefCoreError;
    fn try_from(n: Number) -> Result<Self> {
        if n.type_id != TypeID::Float {
            return Err(n.type_mismatch_error(TypeID::Float));
        }

        if n.overflows(BitSize::Double) {
            return Err(n.overflow_error(BitSize::Double));
        }

        match n.inner {
            Some(inner) => Ok(f64::from_be_bytes(inner)),
            None => Err(StefCoreError::IsNull),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- type_byte ---

    #[test]
    fn type_byte_uint_mini_non_nullable() {
        let n = Number::uint(0, BitSize::Mini);
        assert_eq!(n.type_byte(), 0b0_00_00001);
    }

    #[test]
    fn type_byte_uint_double_nullable() {
        let n = Number::uint(0, BitSize::Double).nullable().unwrap();
        assert_eq!(n.type_byte(), 0b1_11_00001);
    }

    #[test]
    fn type_byte_float_single() {
        let n = Number::float_single(0.0);
        assert_eq!(n.type_byte(), 0b0_10_00011);
    }

    // --- serialize ---

    #[test]
    fn serialize_u8_value_17() {
        let n = Number::uint(17, BitSize::Mini);
        assert_eq!(n.serialize().unwrap(), vec![0b0_00_00001, 0x11]);
    }

    #[test]
    fn serialize_u32_value_256() {
        let n = Number::uint(256, BitSize::Single);
        assert_eq!(n.serialize().unwrap(), vec![0b0_10_00001, 0x00, 0x00, 0x01, 0x00]);
    }

    #[test]
    fn serialize_nullable_present() {
        let n = Number::uint(17, BitSize::Mini).nullable().unwrap();
        assert_eq!(n.serialize().unwrap(), vec![0b1_00_00001, 0x01, 0x11]);
    }

    #[test]
    fn serialize_null() {
        let n = Number::null(BitSize::Mini, TypeID::Uint);
        assert_eq!(n.serialize().unwrap(), vec![0b1_00_00001, 0x00]);
    }

    #[test]
    fn serialize_f64() {
        let value = 1.0f64;
        let n = Number::float_double(value);
        let mut expected = vec![0b0_11_00011];
        expected.extend_from_slice(&value.to_be_bytes());
        assert_eq!(n.serialize().unwrap(), expected);
    }

    // --- deserialize ---

    #[test]
    fn deserialize_u8() {
        let bytes = vec![0b0_00_00001, 0x11];
        let n = Number::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert_eq!(u8::try_from(n).unwrap(), 17u8);
    }

    #[test]
    fn deserialize_nullable_present() {
        let bytes = vec![0b1_00_00001, 0x01, 0x11];
        let n = Number::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(n.is_nullable());
        assert!(!n.is_null());
        assert_eq!(u8::try_from(n).unwrap(), 17u8);
    }

    #[test]
    fn deserialize_null() {
        let bytes = vec![0b1_00_00001, 0x00];
        let n = Number::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(n.is_null());
        assert!(matches!(u8::try_from(n), Err(StefCoreError::IsNull)));
    }

    // --- TryFrom ---

    #[test]
    fn try_from_type_mismatch() {
        let n = Number::uint(1, BitSize::Mini);
        assert!(matches!(i8::try_from(n), Err(StefCoreError::TypeMismatch { .. })));
    }

    #[test]
    fn try_from_size_overflow() {
        let n = Number::uint(1000, BitSize::Double);
        assert!(matches!(u8::try_from(n), Err(StefCoreError::SizeOverflow { .. })));
    }

    #[test]
    fn try_from_f32_roundtrip() {
        let value = 3.14f32;
        let n = Number::float_single(value);
        assert_eq!(f32::try_from(n).unwrap(), value);
    }

    #[test]
    fn try_from_f64_roundtrip() {
        let value = std::f64::consts::PI;
        let n = Number::float_double(value);
        assert_eq!(f64::try_from(n).unwrap(), value);
    }

    #[test]
    fn try_from_i64_negative() {
        let n = Number::int(-1000, BitSize::Double);
        assert_eq!(i64::try_from(n).unwrap(), -1000i64);
    }

    // --- nullable/non_nullable builders ---

    #[test]
    fn nullable_builder() {
        let n = Number::uint(5, BitSize::Mini).nullable().unwrap();
        assert!(n.is_nullable());
        assert!(!n.is_null());
    }

    #[test]
    fn non_nullable_builder() {
        let n = Number::uint(5, BitSize::Mini).nullable().unwrap().non_nullable().unwrap();
        assert!(!n.is_nullable());
    }

    // --- null constructor panics on the wrong type ---

    #[test]
    #[should_panic]
    fn null_panics_on_invalid_type_id() {
        Number::null(BitSize::Mini, TypeID::Bool);
    }
}