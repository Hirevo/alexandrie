use std::io::{self, Read};

use semver::Version;
use serde::{Deserialize, Serialize};

pub mod disk;

use crate::error::Error;
use crate::storage::disk::DiskStorage;

/// The crate storage strategy enum type.  
///
/// It represents which storage strategy is currently used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Storage {
    /// Local on-disk crate storage.
    #[serde(rename = "disk")]
    DiskStorage(DiskStorage),
    // S3Storage(...),
}

/// The required trait that any storage type must implement.
pub trait Store {
    /// Retrieves a crate tarball from the store.
    fn get_crate(&self, name: &str, version: Version) -> Result<Vec<u8>, Error>;
    /// Reads a crate tarball from the store.
    fn read_crate(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        struct Reader {
            source: Vec<u8>,
        }

        impl Read for Reader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.source.as_slice().read(buf)
            }
        }

        Ok(Box::new(Reader {
            source: self.get_crate(name, version)?,
        }))
    }
    /// Save a new crate tarball into the store.
    fn store_crate(&self, name: &str, version: Version, data: impl Read) -> Result<(), Error>;
}

impl Store for Storage {
    fn get_crate(&self, name: &str, version: Version) -> Result<Vec<u8>, Error> {
        match self {
            Storage::DiskStorage(storage) => storage.get_crate(name, version),
        }
    }

    fn read_crate(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        match self {
            Storage::DiskStorage(storage) => storage.read_crate(name, version),
        }
    }

    fn store_crate(&self, name: &str, version: Version, data: impl Read) -> Result<(), Error> {
        match self {
            Storage::DiskStorage(storage) => storage.store_crate(name, version, data),
        }
    }
}
