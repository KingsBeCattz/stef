use std::collections::BTreeMap;
use std::io::Read;
use crate::error::StefCoreError;
use crate::value::{ReadOptions, StefValue, Value, utils};
use crate::value::bitsize::BitSize;
use crate::value::typeid::TypeID;
use crate::constants;
use crate::types::{TypeByte, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Record {
    pub fields: Option<BTreeMap<String, Value>>,
    nullable: bool,
}

impl Record {
    pub fn new_nullable(fields: BTreeMap<String, Value>, nullable: bool) -> Self {
        Self { fields: Some(fields), nullable }
    }
    pub fn new(fields: BTreeMap<String, Value>) -> Self {
        Self::new_nullable(fields, false)
    }
    pub fn new_empty() -> Self {
        Self { fields: Some(BTreeMap::new()), nullable: false }
    }
    pub fn null() -> Self {
        Self { fields: None, nullable: true }
    }
    pub fn fields_mut(&mut self) -> Option<&mut BTreeMap<String, Value>> {
        match self.fields {
            Some(ref mut fields) => Some(fields),
            None => None,
        }
    }
}

impl From<BTreeMap<String, Value>> for Record {
    fn from(fields: BTreeMap<String, Value>) -> Self {
        Self::new(fields)
    }
}

impl From<Record> for BTreeMap<String, Value> {
    fn from(record: Record) -> Self {
        record.fields.unwrap_or(BTreeMap::new())
    }
}

impl StefValue for Record {
    type Error = StefCoreError;

    fn type_id(&self) -> TypeID {
        TypeID::Record
    }

    fn bit_size(&self) -> Option<BitSize> {
        if let Some(fields) = &self.fields {
            Some(BitSize::find_size(fields.len() as u64))
        } else {
            None
        }
    }

    fn is_null(&self) -> bool {
        self.fields.is_none()
    }

    fn is_nullable(&self) -> bool {
        self.nullable
    }

    fn nullable(self) -> Result<Self> {
        Ok(Self { nullable: true, ..self })
    }

    fn non_nullable(self) -> Result<Self> {
        Ok(Self { nullable: false, ..self })
    }

    fn type_byte(&self) -> TypeByte {
        let null_byte: u8 = if self.nullable { constants::reading::NULLABLE_MASK } else { 0 };
        null_byte | self.bit_size().unwrap_or(BitSize::Mini) as u8 | self.type_id() as u8
    }

    fn serialize(&self) -> Result<Vec<u8>> {
        let type_byte = self.type_byte();
        let mut bytes: Vec<u8> = vec![type_byte];

        if self.nullable {
            bytes.push(self.fields.is_some() as u8);
        }

        if let Some(fields) = &self.fields {
            utils::write_length(&mut bytes, fields.len() as u64, self.bit_size().unwrap());

            for (name, value) in fields.iter() {
                if name.len() > constants::writing::MAX_NAME_LENGTH {
                    return Err(Self::Error::NameTooLong {
                        max: constants::writing::MAX_NAME_LENGTH as u8,
                        actual: name.len(),
                    })
                }
                bytes.push(name.len() as u8);
                bytes.extend_from_slice(&name.as_bytes());
                bytes.extend_from_slice(&value.serialize()?);
            }
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
        let nullable = (type_byte & constants::reading::NULLABLE_MASK) != 0;

        let type_id =
            TypeID::from_bits(type_byte).ok_or(Self::Error::InvalidTypeId(type_byte))?;

        if type_id != TypeID::Record {
            return Err(Self::Error::CannotReadAs(type_id));
        }

        let size = BitSize::from_bits(type_byte);

        if nullable {
            let mut presence = [0u8; 1];
            bytes.read_exact(&mut presence).map_err(StefCoreError::Io)?;

            if presence[0] == 0x00 {
                return Ok( Self {
                    fields: None,
                    nullable
                } );
            }
        }

        let length = utils::read_length(bytes, size).map_err(Self::Error::Io)?;

        let mut fields: BTreeMap<String, Value> = BTreeMap::new();
        for _ in 0..length {
            let mut name_length = [0u8; 1];
            bytes
                .read_exact(&mut name_length)
                .map_err(Self::Error::Io)?;
            let name_length = u8::from(name_length[0]);
            let mut name = vec![0u8; name_length as usize];
            bytes
                .read_exact(&mut name)
                .map_err(Self::Error::Io)?;
            fields.insert(String::from_utf8(name).map_err(|_| Self::Error::InvalidUtf8)?, Value::deserialize(&mut *bytes, depth, options)?);
        }

        Ok( Self {
            fields: Some(fields),
            nullable
        } )
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::io::Cursor;
    use crate::value::{StefValue, Value, ReadOptions};
    use crate::value::bitsize::BitSize;
    use crate::value::number::Number;
    use crate::value::string::Text;
    use super::Record;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn round_trip(record: Record) -> Record {
        let bytes = record.serialize().expect("serialize failed");
        let mut cursor = Cursor::new(bytes);
        Record::deserialize(&mut cursor, 0, None).expect("deserialize failed")
    }

    fn single_field(name: &str, value: Value) -> BTreeMap<String, Value> {
        let mut fields = BTreeMap::new();
        fields.insert(name.to_string(), value);
        fields
    }

    // ── Construction ─────────────────────────────────────────────────────────

    #[test]
    fn new_record_is_not_nullable_and_has_fields() {
        let record = Record::new(BTreeMap::new());
        assert!(!record.is_nullable());
        assert!(!record.is_null());
        assert!(record.fields.is_some());
    }

    #[test]
    fn null_record_is_nullable_and_has_no_fields() {
        let record = Record::null();
        assert!(record.is_nullable());
        assert!(record.is_null());
        assert!(record.fields.is_none());
    }

    #[test]
    fn new_empty_record_has_zero_fields() {
        let record = Record::new_empty();
        assert_eq!(record.fields.unwrap().len(), 0);
    }

    // ── Serialization ────────────────────────────────────────────────────────

    #[test]
    fn empty_record_serializes_correctly() {
        let record = Record::new_empty();
        let bytes = record.serialize().unwrap();

        assert_eq!(bytes[0] & 0b1000_0000, 0);  // not nullable
        assert_eq!(bytes[1], 0x00);              // field count = 0
    }

    #[test]
    fn nullable_null_record_serializes_with_presence_byte_zero() {
        let record = Record::null();
        let bytes = record.serialize().unwrap();

        assert_eq!(bytes[0] & 0b1000_0000, 0b1000_0000); // nullable bit set
        assert_eq!(bytes[1], 0x00);                       // presence = null
        assert_eq!(bytes.len(), 2);
    }

    #[test]
    fn nullable_present_record_serializes_with_presence_byte_one() {
        let record = Record::new_nullable(BTreeMap::new(), true);
        let bytes = record.serialize().unwrap();

        assert_eq!(bytes[0] & 0b1000_0000, 0b1000_0000); // nullable bit set
        assert_eq!(bytes[1], 0x01);                       // presence = has value
    }

    // ── Round-trip ───────────────────────────────────────────────────────────

    #[test]
    fn round_trip_empty_record() {
        let record = Record::new_empty();
        assert_eq!(record, round_trip(record.clone()));
    }

    #[test]
    fn round_trip_nullable_null_record() {
        let record = Record::null();
        assert_eq!(record, round_trip(record.clone()));
    }

    #[test]
    fn round_trip_nullable_present_record() {
        let record = Record::new_nullable(BTreeMap::new(), true);
        assert_eq!(record, round_trip(record.clone()));
    }

    #[test]
    fn round_trip_record_with_uint_field() {
        let record = Record::new(single_field(
            "age",
            Value::Number(Number::uint(17, BitSize::Mini)),
        ));
        assert_eq!(record, round_trip(record.clone()));
    }

    #[test]
    fn round_trip_record_with_multiple_fields() {
        let mut fields = BTreeMap::new();
        fields.insert("age".to_string(),   Value::Number(Number::uint(17, BitSize::Mini)));
        fields.insert("name".to_string(),  Value::Text(Text::new("Johan")));
        fields.insert("score".to_string(), Value::Number(Number::float_double(9.5)));

        let record = Record::new(fields);
        assert_eq!(record, round_trip(record.clone()));
    }

    #[test]
    fn round_trip_nested_record() {
        let inner = Record::new(single_field(
            "x",
            Value::Number(Number::uint(1, BitSize::Mini)),
        ));
        let record = Record::new(single_field("inner", Value::Record(inner)));
        assert_eq!(record, round_trip(record.clone()));
    }

    // ── Field ordering ───────────────────────────────────────────────────────

    #[test]
    fn fields_are_preserved_in_alphabetical_order_after_round_trip() {
        let mut fields = BTreeMap::new();
        fields.insert("zebra".to_string(), Value::Number(Number::uint(1, BitSize::Mini)));
        fields.insert("apple".to_string(), Value::Number(Number::uint(2, BitSize::Mini)));
        fields.insert("mango".to_string(), Value::Number(Number::uint(3, BitSize::Mini)));

        let result = round_trip(Record::new(fields));
        let keys: Vec<&String> = result.fields.as_ref().unwrap().keys().collect();
        assert_eq!(keys, vec!["apple", "mango", "zebra"]);
    }

    // ── Security limits ──────────────────────────────────────────────────────

    #[test]
    fn exceeding_max_depth_returns_error() {
        let inner = Record::new(single_field(
            "x",
            Value::Number(Number::uint(1, BitSize::Mini)),
        ));
        let outer = Record::new(single_field("inner", Value::Record(inner)));

        let bytes = outer.serialize().unwrap();
        let mut cursor = Cursor::new(bytes);

        let options = ReadOptions { max_depth: Some(1), ..Default::default() };
        let result = Record::deserialize(&mut cursor, 0, Some(&options));

        assert!(result.is_err());
    }
}