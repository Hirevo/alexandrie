use std::io::Read;

use serde::{Serialize, Deserialize};
use semver::Version;

mod disk;
pub use disk::*;

use crate::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Storage {
    DiskStorage(DiskStorage),
    // S3Storage(...),
}

pub trait Store {
    fn get_crate(&self, name: &str, version: Version) -> Result<Vec<u8>, Error>;
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
