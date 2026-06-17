use crate::constants;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct ReadOptions {
    pub max_depth: Option<usize>,
    pub max_bytes: Option<u64>,
    pub max_elements: Option<u64>,
    pub max_fields: Option<u64>,
}

impl Default for ReadOptions {
    fn default() -> Self {
        Self {
            max_depth: Some(constants::reading::DEFAULT_MAX_DEPTH),
            max_bytes: Some(constants::reading::DEFAULT_MAX_BYTES),
            max_elements: Some(constants::reading::DEFAULT_MAX_ELEMENTS),
            max_fields: Some(constants::reading::DEFAULT_MAX_FIELDS),
        }
    }
}