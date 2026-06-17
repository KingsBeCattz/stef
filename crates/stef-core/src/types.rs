use std::collections::BTreeMap;
use crate::value;

pub type TypeByte = u8;

pub type Result<T> = std::result::Result<T, crate::error::StefCoreError>;
pub type TopLevelRecord = BTreeMap<String, value::Value>;