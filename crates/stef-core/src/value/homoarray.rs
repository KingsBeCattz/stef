use std::io::Read;
use crate::error::StefCoreError;
use crate::value::{ReadOptions, StefValue, Value};
use crate::value::bitsize::BitSize;
use crate::value::typeid::TypeID;
use crate::value::utils;
use crate::types::{TypeByte, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct HomoArray {
    elements: Vec<Value>,
    element_type_byte: TypeByte
}

impl HomoArray {
    pub fn new(elements: Vec<Value>, element_type_byte: TypeByte) -> Self {
        Self { elements, element_type_byte }
    }
    pub fn new_empty(element_type_byte: TypeByte) -> Self {
        Self { elements: vec![], element_type_byte }
    }
    pub fn element_type_byte(&self) -> TypeByte {
        self.element_type_byte
    }
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }
    pub fn elements_iter_mut(&mut self) -> impl Iterator<Item = &mut Value> {
        self.elements.iter_mut()
    }
    pub fn len(&self) -> usize {
        self.elements.len()
    }
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn push(&mut self, element: Value) -> Result<()> {
        if self.element_type_byte != element.type_byte() {
            return Err(StefCoreError::TypeByteMismatch {
                expected: self.element_type_byte,
                found: element.type_byte()
            });
        }

        if self.elements.len() >= isize::MAX as usize {
            return Err(StefCoreError::ExceededMaxArraySize(isize::MAX));
        }

        self.elements.push(element);
        Ok(())
    }
    pub fn pop(&mut self) -> Option<Value> {
        self.elements.pop()
    }

    pub fn remove(&mut self, index: usize) -> Option<Value> {
        if index >= self.elements.len() {
            return None;
        }
        Some(self.elements.remove(index))
    }

    pub fn insert(&mut self, index: usize, element: Value) -> Result<()> {
        if self.element_type_byte != element.type_byte() {
            return Err(StefCoreError::TypeByteMismatch {
                expected: self.element_type_byte,
                found: element.type_byte()
            });
        }
        if index > self.elements.len() {
            return Err(StefCoreError::IndexOutOfBounds {
                index, max: self.elements.len()
            });
        }
        self.elements.insert(index, element);
        Ok(())
    }
}

impl StefValue for HomoArray {
    type Error = StefCoreError;

    fn type_id(&self) -> TypeID {
        TypeID::HomoArray
    }

    fn bit_size(&self) -> Option<BitSize> {
        Some(BitSize::find_size(self.elements.len() as u64))
    }

    fn nullable(self) -> Result<Self> {
        Err(Self::Error::NullableNotSupported(TypeID::HomoArray))
    }

    fn non_nullable(self) -> Result<Self> {
        Ok(self)
    }

    fn type_byte(&self) -> TypeByte {
        self.bit_size().unwrap() as u8 | TypeID::HomoArray as u8
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let type_byte = self.type_byte();
        let mut bytes = vec![type_byte];
        utils::write_length(&mut bytes, self.elements.len() as u64, self.bit_size().unwrap());
        bytes.push(self.element_type_byte);
        for element in &self.elements {
            let element_ser = element.serialize()?;
            let element_ser = &element_ser[1..];
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
            .map_err(StefCoreError::Io)?;

        let type_byte: TypeByte = type_byte[0];

        let type_id =
            TypeID::from_bits(type_byte).ok_or(StefCoreError::InvalidTypeId(type_byte))?;

        if type_id != TypeID::HomoArray {
            return Err(StefCoreError::CannotReadAs(type_id));
        }

        let size = BitSize::from_bits(type_byte);

        let length = utils::read_length(bytes, size).map_err(StefCoreError::Io)?;

        let mut elements_type_byte = [0u8; 1];

        bytes
            .read_exact(&mut elements_type_byte)
            .map_err(StefCoreError::Io)?;
        let element_type_byte: TypeByte = elements_type_byte[0];

        let mut elements: Vec<Value> = vec![];
        for _ in 0..length {
            let type_byte_buf = [element_type_byte];
            let mut element_reader = type_byte_buf.as_slice().chain(&mut *bytes);
            elements.push(Value::deserialize(&mut element_reader, depth, options)?);
        }

        Ok( Self {
            elements,
            element_type_byte
        } )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::number::Number;
    use crate::value::bitsize::BitSize;

    fn uint_value(n: u64, size: BitSize) -> Value {
        Value::Number(Number::uint(n, size))
    }

    // --- constructors ---

    #[test]
    fn new_empty_creates_empty_array() {
        let arr = HomoArray::new_empty(0x01); // uint Mini type byte
        assert!(arr.is_empty());
        assert_eq!(arr.len(), 0);
    }

    #[test]
    fn new_with_elements() {
        let elements = vec![uint_value(1, BitSize::Mini), uint_value(2, BitSize::Mini)];
        let arr = HomoArray::new(elements, 0x01);
        assert_eq!(arr.len(), 2);
    }

    // --- push ---

    #[test]
    fn push_valid_element() {
        let mut arr = HomoArray::new_empty(0x01);
        assert!(arr.push(uint_value(42, BitSize::Mini)).is_ok());
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn push_type_mismatch_returns_error() {
        let mut arr = HomoArray::new_empty(0x01); // uint
        let bool_val = Value::Bool(crate::value::bool::Bool::new(true));
        assert!(matches!(
            arr.push(bool_val),
            Err(StefCoreError::TypeByteMismatch { .. })
        ));
    }

    // --- insert ---

    #[test]
    fn insert_valid_element() {
        let mut arr = HomoArray::new(vec![uint_value(1, BitSize::Mini), uint_value(3, BitSize::Mini)], 0x01);
        assert!(arr.insert(1, uint_value(2, BitSize::Mini)).is_ok());
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn insert_out_of_bounds_returns_error() {
        let mut arr = HomoArray::new_empty(0x01);
        assert!(arr.insert(5, uint_value(1, BitSize::Mini)).is_err());
    }

    // --- pop / remove ---

    #[test]
    fn pop_returns_last_element() {
        let mut arr = HomoArray::new(vec![uint_value(1, BitSize::Mini), uint_value(2, BitSize::Mini)], 0x01);
        assert!(arr.pop().is_some());
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn remove_valid_index() {
        let mut arr = HomoArray::new(vec![uint_value(1, BitSize::Mini), uint_value(2, BitSize::Mini)], 0x01);
        assert!(arr.remove(0).is_some());
        assert_eq!(arr.len(), 1);
    }

    #[test]
    fn remove_invalid_index_returns_none() {
        let mut arr = HomoArray::new_empty(0x01);
        assert!(arr.remove(5).is_none());
    }

    // --- serialize ---

    #[test]
    fn serialize_empty_array() {
        let arr = HomoArray::new_empty(0x01);
        assert_eq!(arr.serialize().unwrap(), vec![
            0b0_00_10001, // type byte: non-nullable, Mini, HomoArray
            0x00,         // count = 0
            0x01,         // element type byte (uint Mini)
        ]);
    }

    #[test]
    fn serialize_three_u8_elements() {
        let elements = vec![
            uint_value(1, BitSize::Mini),
            uint_value(2, BitSize::Mini),
            uint_value(3, BitSize::Mini),
        ];
        let arr = HomoArray::new(elements, 0x01);
        assert_eq!(arr.serialize().unwrap(), vec![
            0b0_00_10001, // type byte: non-nullable, Mini, HomoArray
            0x03,         // count = 3
            0x01,         // element type byte (uint Mini)
            0x01, 0x02, 0x03 // payloads without type byte
        ]);
    }

    // --- deserialize ---

    #[test]
    fn deserialize_empty_array() {
        let bytes = vec![0b0_00_10001, 0x00, 0x01];
        let arr = HomoArray::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert!(arr.is_empty());
    }

    #[test]
    fn deserialize_three_u8_elements() {
        let bytes = vec![
            0b0_00_10001,
            0x03,
            0x01,
            0x01, 0x02, 0x03
        ];
        let arr = HomoArray::deserialize(&mut bytes.as_slice(), 0, None).unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn deserialize_wrong_type_id() {
        let bytes = vec![0b0_00_00001, 0x01]; // uint, no HomoArray
        assert!(matches!(
            HomoArray::deserialize(&mut bytes.as_slice(), 0, None),
            Err(StefCoreError::CannotReadAs(_))
        ));
    }

    #[test]
    fn deserialize_max_depth_exceeded() {
        let options = ReadOptions { max_depth: Some(0), ..Default::default() };
        let bytes = vec![0b0_00_10001, 0x00, 0x01];
        assert!(matches!(
            HomoArray::deserialize(&mut bytes.as_slice(), 0, Some(&options)),
            Err(StefCoreError::MaximumDepthExceeded { .. })
        ));
    }

    // --- roundtrip ---

    #[test]
    fn roundtrip() {
        let elements = vec![
            uint_value(10, BitSize::Mini),
            uint_value(20, BitSize::Mini),
            uint_value(30, BitSize::Mini),
        ];
        let arr = HomoArray::new(elements, 0x01);
        let serialized = arr.serialize().unwrap();
        let deserialized = HomoArray::deserialize(&mut serialized.as_slice(), 0, None).unwrap();
        assert_eq!(deserialized.len(), 3);
    }
}