use std::io::{Cursor, ErrorKind, Read};
use crc32fast::Hasher;

use crate::error::StefCoreError;
use crate::flags::Flags;
use crate::types::TopLevelRecord;
use crate::types::Result;
use crate::value::StefValue;

pub mod value;
pub mod error;
pub mod readoptions;
pub mod types;
pub mod flags;
pub(crate) mod constants;



#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct File {
    pub flags: Flags,
    pub root: TopLevelRecord,
    pub meta: Option<TopLevelRecord>,
}

impl File {
    pub fn new(root: TopLevelRecord, meta: Option<TopLevelRecord>) -> Self {
        File {
            flags: Flags::default(),
            root,
            meta,
        }
    }
    pub fn new_empty() -> Self {
        File {
            flags: Flags::default(),
            root: TopLevelRecord::new(),
            meta: None,
        }
    }
    pub fn set_flags(&mut self, flags: Flags) {
        self.flags = flags;
    }
    pub fn get_root_mut(&mut self) -> &mut TopLevelRecord {
        &mut self.root
    }
    pub fn get_meta_mut(&mut self) -> Option<&mut TopLevelRecord> {
        self.meta.as_mut()
    }

    fn serialize_top_level_record(name: &str, record: &TopLevelRecord) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        buffer.push(name.len() as u8);
        buffer.extend_from_slice(name.as_bytes());

        buffer.extend_from_slice((record.len() as u16).to_be_bytes().as_slice());

        for (name, value) in record.iter() {
            if name.len() > constants::writing::MAX_NAME_LENGTH {
                return Err(StefCoreError::NameTooLong {
                    max: constants::writing::MAX_NAME_LENGTH as u8,
                    actual: name.len(),
                })
            }

            buffer.push(name.len() as u8);
            buffer.extend_from_slice(name.as_bytes());
            buffer.extend_from_slice(&value.serialize()?);
        }

        Ok(buffer)
    }

    fn deserialize_top_level_record(buffer: &mut impl Read, expected_name: &str) -> Result<TopLevelRecord> {
        let mut name_length = [0u8; 1];
        buffer.read_exact(&mut name_length).map_err(StefCoreError::Io)?;
        let name_length = name_length[0] as usize;
        let mut name = vec![0u8; name_length];
        buffer.read_exact(&mut name).map_err(StefCoreError::Io)?;
        let name = String::from_utf8(name).map_err(|_| StefCoreError::InvalidUtf8)?;
        if name != expected_name {
            return Err(StefCoreError::UnexpectedTopLevelRecord(name))
        };
        let mut fields_count = [0u8; 2];
        buffer.read_exact(&mut fields_count).map_err(StefCoreError::Io)?;

        let fields_count = u16::from_be_bytes(fields_count) as usize;
        let mut record = TopLevelRecord::new();

        for _ in 0..fields_count {
            let mut name_length = [0u8; 1];
            buffer.read_exact(&mut name_length).map_err(StefCoreError::Io)?;
            let name_length = name_length[0] as usize;

            let mut name = vec![0u8; name_length];
            buffer.read_exact(&mut name).map_err(StefCoreError::Io)?;

            let name = String::from_utf8(name).map_err(|_| StefCoreError::InvalidUtf8)?;
            let value = StefValue::deserialize(buffer, 0, None)?;
            record.insert(name, value);
        }

        Ok(record)
    }

    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        buffer.extend_from_slice(constants::MAGIC);
        buffer.push(constants::VERSION);
        buffer.push((&self.flags).into());

        let mut buffer_content = Vec::new();
        buffer_content.extend_from_slice(Self::serialize_top_level_record(constants::ROOT_NAME, &self.root)?.as_slice());
        if let Some(meta) = &self.meta {
            buffer_content.extend_from_slice(Self::serialize_top_level_record(constants::META_NAME, meta)?.as_slice());
        }

        if self.flags.compressed {
            buffer_content = zstd::encode_all(&mut Cursor::new(buffer_content), 3).map_err(StefCoreError::Io)?;
        }

        if self.flags.checksum {
            let mut hasher = Hasher::new();
            hasher.update(&buffer_content);
            let checksum = hasher.finalize();
            buffer.extend_from_slice(&checksum.to_be_bytes());
        }

        buffer.extend_from_slice(&buffer_content);

        Ok(buffer)
    }

    pub fn deserialize(buffer: &mut impl Read) -> Result<Self> {
        let mut magic = [0u8; 4];
        buffer.read_exact(&mut magic).map_err(StefCoreError::Io)?;
        if &magic != constants::MAGIC {
            return Err(StefCoreError::InvalidMagic);
        }
        let mut version = [0u8; 1];
        buffer.read_exact(&mut version).map_err(StefCoreError::Io)?;

        if version[0] > constants::VERSION {
            return Err(StefCoreError::UnsupportedVersion(version[0]));
        }

        let mut flags = [0u8;1];
        buffer.read_exact(&mut flags).map_err(StefCoreError::Io)?;
        let flags = Flags::from(flags[0]);

        let mut checksum = [0u8;4];
        if flags.checksum {
            buffer.read_exact(&mut checksum).map_err(StefCoreError::Io)?;
        };

        let mut payload = Vec::new();
        buffer.read_to_end(&mut payload).map_err(StefCoreError::Io)?;

        if flags.checksum {
            let expected_checksum = u32::from_be_bytes(checksum);
            let mut hasher = Hasher::new();
            hasher.update(&payload);
            let calculated = hasher.finalize();

            if calculated != expected_checksum {
                return Err(StefCoreError::ChecksumMismatch);
            }
        }

        if flags.compressed {
            payload = zstd::decode_all(&mut Cursor::new(&payload)).map_err(StefCoreError::Io)?;
        };
        let mut payload = Cursor::new(payload);
        let root = Self::deserialize_top_level_record(&mut payload, constants::ROOT_NAME)?;
        let meta = match Self::deserialize_top_level_record(&mut payload, constants::META_NAME) {
            Ok(meta) => Some(meta),
            Err(error) => match error {
                StefCoreError::Io(err) => match err.kind() {
                    ErrorKind::UnexpectedEof => None,
                    _ => return Err(StefCoreError::Io(err)),
                },
                _ => return Err(error),
            }
        };

        Ok(File { flags, root, meta })
    }
}