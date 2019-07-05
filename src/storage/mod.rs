use std::io::Read;

use semver::Version;
use serde::{Deserialize, Serialize};

mod disk;
pub use disk::*;

use crate::Error;

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
    /// Save a new crate tarball into the store.
    fn store_crate(&self, name: &str, version: Version, data: impl Read) -> Result<(), Error>;
}

impl Store for Storage {
    fn get_crate(&self, name: &str, version: Version) -> Result<Vec<u8>, Error> {
        match self {
            Storage::DiskStorage(storage) => storage.get_crate(name, version),
        }
    }

    fn store_crate(&self, name: &str, version: Version, data: impl Read) -> Result<(), Error> {
        match self {
            Storage::DiskStorage(storage) => storage.store_crate(name, version, data),
        }
    }
}
