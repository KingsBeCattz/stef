use std::io::Read;
use crate::error::StefCoreError;
use crate::value::{utils, ReadOptions, StefValue, Value};
use crate::value::bitsize::BitSize;
use crate::value::typeid::TypeID;
use crate::types::{TypeByte, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HeteroArray {
    pub elements: Vec<Value>,
}

impl HeteroArray {
    pub fn new(elements: Vec<Value>) -> Self {
        Self { elements }
    }
    pub fn new_empty() -> Self {
        Self { elements: vec![] }
    }
    pub fn elements_mut(&mut self) -> &mut Vec<Value> {
        &mut self.elements
    }
}

impl From<Vec<Value>> for HeteroArray {
    fn from(elements: Vec<Value>) -> Self {
        Self::new(elements)
    }
}

impl From<HeteroArray> for Vec<Value> {
    fn from(hetero_array: HeteroArray) -> Self {
        hetero_array.elements
    }
}

impl StefValue for HeteroArray {
    type Error = StefCoreError;

    fn type_id(&self) -> TypeID {
        TypeID::HeteroArray
    }

    fn bit_size(&self) -> Option<BitSize> {
        Some(BitSize::find_size(self.elements.len() as u64))
    }

    fn nullable(self) -> Result<Self> {
        Err(Self::Error::NullableNotSupported(TypeID::HeteroArray))
    }

    fn non_nullable(self) -> Result<Self> {
        Ok(self)
    }

    fn type_byte(&self) -> TypeByte {
        self.bit_size().unwrap() as u8 | TypeID::HeteroArray as u8
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let type_byte = self.type_byte();
        let mut bytes = vec![type_byte];
        utils::write_length(&mut bytes, self.elements.len() as u64, self.bit_size().unwrap());
        for element in &self.elements {
            let element_ser = element.serialize()?;
            let element_ser = element_ser.as_slice();
            bytes.extend_from_slice(element_ser);
        }
        Ok(bytes)
    }

    fn deserialize(bytes: &mut impl Read, depth: usize, options: Option<&ReadOptions>) -> Result<Self> {
        let depth = depth + 1;
        if let Some(options) = options && let Some(max_depth) = options.max_depth && depth > max_depth {
            return Err(Self::Error::MaximumDepthExceeded {
                depth, max_depth
            });
        }

        let mut type_byte = [0u8; 1];
        bytes
            .read_exact(&mut type_byte)
            .map_err(Self::Error::Io)?;

        let type_byte: TypeByte = type_byte[0];

        let type_id =
            TypeID::from_bits(type_byte).ok_or(Self::Error::InvalidTypeId(type_byte))?;

        if type_id != TypeID::HeteroArray {
            return Err(Self::Error::CannotReadAs(type_id));
        }

        let size = BitSize::from_bits(type_byte);

        let length = utils::read_length(bytes, size).map_err(Self::Error::Io)?;

        let mut elements: Vec<Value> = vec![];
        for _ in 0..length {
            elements.push(Value::deserialize(&mut *bytes, depth, options)?);
        }

        Ok( Self {
            elements
        } )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::number::Number;
    use crate::value::bitsize::BitSize;
    use crate::value::bool::Bool;
    use crate::error::StefCoreError;

    // type byte: non-nullable, size=Mini(00), HeteroArray(0x10) => 0b0_00_10000 = 0x10
    const HETERO_ARRAY_TYPE_BYTE: u8 = 0b0_00_10000;

    // element type bytes
    const UINT_MINI_TYPE_BYTE: u8 = 0b0_00_00001; // non-nullable, Mini, Uint
    const BOOL_TYPE_BYTE: u8      = 0b0_00_00100; // non-nullable, Mini, Bool

    fn uint_value(n: u64, size: BitSize) -> Value {
        Value::Number(Number::uint(n, size))
    }

    fn bool_value(b: bool) -> Value {
        Value::Bool(Bool::new(b))
    }

    // --- constructors ---

    #[test]
    fn new_empty_creates_empty_array() {
        let arr = HeteroArray::new_empty();
        assert!(arr.elements.is_empty());
    }

    #[test]
    fn new_with_elements() {
        let elements = vec![uint_value(1, BitSize::Mini), bool_value(true)];
        let arr = HeteroArray::new(elements);
        assert_eq!(arr.elements.len(), 2);
    }

    #[test]
    fn from_vec_roundtrip() {
        let elements = vec![uint_value(1, BitSize::Mini), bool_value(false)];
        let arr = HeteroArray::from(elements.clone());
        let back: Vec<Value> = arr.into();
        assert_eq!(back.len(), elements.len());
    }

    // --- elements_mut ---

    #[test]
    fn elements_mut_allows_direct_modification() {
        let mut arr = HeteroArray::new_empty();
        arr.elements_mut().push(uint_value(99, BitSize::Mini));
        assert_eq!(arr.elements.len(), 1);
    }

    // --- serialize ---

    #[test]
    fn serialize_empty_array() {
        let arr = HeteroArray::new_empty();
        assert_eq!(arr.serialize().unwrap(), vec![
            HETERO_ARRAY_TYPE_BYTE, // type byte
            0x00,                   // count = 0
        ]);
    }

    #[test]
    fn serialize_single_uint_element() {
        let arr = HeteroArray::new(vec![uint_value(42, BitSize::Mini)]);
        assert_eq!(arr.serialize().unwrap(), vec![
            HETERO_ARRAY_TYPE_BYTE, // type byte
            0x01,                   // count = 1
            UINT_MINI_TYPE_BYTE,    // element type byte (uint, Mini)
            0x2A,                   // value = 42
        ]);
    }

    #[test]
    fn serialize_mixed_elements() {
        let arr = HeteroArray::new(vec![
            uint_value(1, BitSize::Mini),
            bool_value(true),
        ]);
        assert_eq!(arr.serialize().unwrap(), vec![
            HETERO_ARRAY_TYPE_BYTE, // type byte
            0x02,                   // count = 2
            UINT_MINI_TYPE_BYTE,    // element 0 type byte
            0x01,                   // element 0 value = 1
            BOOL_TYPE_BYTE,         // element 1 type byte
            0x01,                   // element 1 value = true
        ]);
    }

    // --- deserialize ---

    #[test]
    fn deserialize_empty_array() {
        let arr = HeteroArray::new_empty();
        let serialized = arr.serialize().unwrap();
        let result = HeteroArray::deserialize(&mut serialized.as_slice(), 0, None).unwrap();
        assert!(result.elements.is_empty());
    }

    #[test]
    fn deserialize_wrong_type_id() {
        // type byte for Uint, not HeteroArray
        let bytes = vec![UINT_MINI_TYPE_BYTE, 0x01];
        assert!(matches!(
            HeteroArray::deserialize(&mut bytes.as_slice(), 0, None),
            Err(StefCoreError::CannotReadAs(_))
        ));
    }

    #[test]
    fn deserialize_max_depth_exceeded() {
        let options = ReadOptions { max_depth: Some(0), ..Default::default() };
        let serialized = HeteroArray::new_empty().serialize().unwrap();
        assert!(matches!(
            HeteroArray::deserialize(&mut serialized.as_slice(), 0, Some(&options)),
            Err(StefCoreError::MaximumDepthExceeded { .. })
        ));
    }

    // --- roundtrip ---

    #[test]
    fn roundtrip_empty() {
        let arr = HeteroArray::new_empty();
        let serialized = arr.serialize().unwrap();
        let result = HeteroArray::deserialize(&mut serialized.as_slice(), 0, None).unwrap();
        assert!(result.elements.is_empty());
    }

    #[test]
    fn roundtrip_single_element() {
        let arr = HeteroArray::new(vec![bool_value(true)]);
        let serialized = arr.serialize().unwrap();
        let result = HeteroArray::deserialize(&mut serialized.as_slice(), 0, None).unwrap();
        assert_eq!(result.elements.len(), 1);
    }

    #[test]
    fn roundtrip_mixed_types() {
        let arr = HeteroArray::new(vec![
            uint_value(10, BitSize::Mini),
            bool_value(false),
            uint_value(255, BitSize::Mini),
        ]);
        let serialized = arr.serialize().unwrap();
        let result = HeteroArray::deserialize(&mut serialized.as_slice(), 0, None).unwrap();
        assert_eq!(result.elements.len(), 3);
    }
}