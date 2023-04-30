use crate::error::Error;
use ic_stable_structures::BoundedStorable;
use serde::{Deserialize, Serialize};

pub const FILE_CHUNK_SIZE: usize = 4096;
pub const MAX_FILE_NAME: usize = 255;

// The unique identifier of a node, which can be a file or a directory.
// Also known as inode in WASI and other file systems.
pub type Node = u64;

// An integer type for representing file sizes and offsets.
pub type FileSize = u64;

// An index of a file chunk.
pub type FileChunkIndex = u32;

// A file consists of multiple file chunks.
#[derive(Clone, Debug)]
pub struct FileChunk {
    pub bytes: [u8; FILE_CHUNK_SIZE],
}

impl Default for FileChunk {
    fn default() -> Self {
        Self {
            bytes: [0; FILE_CHUNK_SIZE],
        }
    }
}

impl ic_stable_structures::Storable for FileChunk {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        std::borrow::Cow::Borrowed(&self.bytes)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Self {
            bytes: bytes.as_ref().try_into().unwrap(),
        }
    }
}

impl ic_stable_structures::BoundedStorable for FileChunk {
    const MAX_SIZE: u32 = FILE_CHUNK_SIZE as u32;
    const IS_FIXED_SIZE: bool = true;
}

// Contains metadata of a node.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub node: Node,
    pub file_type: FileType,
    pub link_count: u64,
    pub size: FileSize,
    pub times: Times,
    pub first_dir_entry: Option<DirEntryIndex>,
    pub last_dir_entry: Option<DirEntryIndex>,
}

impl ic_stable_structures::Storable for Metadata {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut buf = vec![];
        ciborium::ser::into_writer(&self, &mut buf).unwrap();
        assert!(Self::MAX_SIZE >= buf.len() as u32);
        std::borrow::Cow::Owned(buf)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref()).unwrap()
    }
}

impl ic_stable_structures::BoundedStorable for Metadata {
    // This value was obtained by printing `Storable::to_bytes().len()`,
    // which is 120 and rounding it up to 128.
    const MAX_SIZE: u32 = 128;
    const IS_FIXED_SIZE: bool = true;
}

// The type of a node.
#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Directory,
    #[default]
    RegularFile,
    SymbolicLink,
}

impl TryFrom<u8> for FileType {
    type Error = Error;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            3 => Ok(FileType::Directory),
            4 => Ok(FileType::RegularFile),
            7 => Ok(FileType::SymbolicLink),
            _ => Err(Error::InvalidFileType),
        }
    }
}

impl Into<u8> for FileType {
    fn into(self) -> u8 {
        match self {
            FileType::Directory => 3,
            FileType::RegularFile => 4,
            FileType::SymbolicLink => 7,
        }
    }
}

// The time stats of a node.
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct Times {
    pub accessed: u64,
    pub modified: u64,
    pub created: u64,
}

// The name of a file or a directory. Most operating systems limit the max file
// name length to 255.
#[derive(Clone, Debug)]
pub struct FileName {
    pub length: u8,
    pub bytes: [u8; MAX_FILE_NAME],
}

impl Default for FileName {
    fn default() -> Self {
        Self {
            length: 0,
            bytes: [0; MAX_FILE_NAME],
        }
    }
}

impl Serialize for FileName {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_bytes::Bytes::new(&self.bytes).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FileName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: Vec<u8> = serde_bytes::deserialize(deserializer).unwrap();
        let length = bytes.len();
        let bytes_array: [u8; MAX_FILE_NAME] = bytes.try_into().or_else(|_| {
            Err(serde::de::Error::invalid_length(
                length,
                &"expected MAX_FILE_NAME bytes",
            ))
        })?;
        Ok(FileName {
            length: length as u8,
            bytes: bytes_array,
        })
    }
}

impl FileName {
    pub fn new(name: &str) -> Result<Self, Error> {
        let name = name.as_bytes();
        let len = name.len();
        if len > MAX_FILE_NAME {
            return Err(Error::NameTooLong);
        }
        let mut bytes = [0; MAX_FILE_NAME];
        (&mut bytes[0..len]).copy_from_slice(name);
        Ok(Self {
            length: len as u8,
            bytes,
        })
    }
}

// An index of a directory entry.
pub type DirEntryIndex = u32;

// A directory contains a list of directory entries.
// Each entry describes a name of a file or a directory.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct DirEntry {
    pub name: FileName,
    pub node: Node,
    pub next_entry: Option<DirEntryIndex>,
    pub prev_entry: Option<DirEntryIndex>,
}

impl ic_stable_structures::Storable for DirEntry {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        let mut buf = vec![];
        ciborium::ser::into_writer(&self, &mut buf).unwrap();
        assert!(Self::MAX_SIZE >= buf.len() as u32);
        std::borrow::Cow::Owned(buf)
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        ciborium::de::from_reader(bytes.as_ref()).unwrap()
    }
}

impl ic_stable_structures::BoundedStorable for DirEntry {
    // This value was obtained by printing `DirEntry::to_bytes().len()`,
    // which is 298 and rounding it up to 304.
    const MAX_SIZE: u32 = 304;
    const IS_FIXED_SIZE: bool = true;
}
