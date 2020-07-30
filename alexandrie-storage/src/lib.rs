use std::io::{self, Read};

use semver::Version;

pub mod config;
/// Local on-disk crate storage mechanism.
pub mod disk;
pub mod error;
/// S3 storage mechanism.
#[cfg(feature = "s3")]
pub mod s3;

use crate::disk::DiskStorage;
use crate::error::Error;

/// The crate storage strategy enum type.  
///
/// It represents which storage strategy is currently used.
#[derive(Debug, Clone)]
pub enum Storage {
    /// Local on-disk crate storage.
    Disk(DiskStorage),

    /// S3 crate storage.
    #[cfg(feature = "s3")]
    S3(s3::S3Storage),
    // TODO: Add a `Store` implementation using a git repository.
    // Git(GitStorage),
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
    fn store_crate(&self, name: &str, version: Version, data: Vec<u8>) -> Result<(), Error>;

    /// Retrieves a rendered README from the store.
    fn get_readme(&self, name: &str, version: Version) -> Result<String, Error>;
    /// Reads a rendered README from the store.
    fn read_readme(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        struct Reader {
            source: Vec<u8>,
        }

        impl Read for Reader {
            fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
                self.source.as_slice().read(buf)
            }
        }

        Ok(Box::new(Reader {
            source: self.get_readme(name, version)?.into_bytes(),
        }))
    }
    /// Stores a new rendered README into the store.
    fn store_readme(&self, name: &str, version: Version, data: String) -> Result<(), Error>;
}

impl Store for Storage {
    fn get_crate(&self, name: &str, version: Version) -> Result<Vec<u8>, Error> {
        match self {
            Storage::Disk(storage) => storage.get_crate(name, version),
            #[cfg(feature = "s3")]
            Storage::S3(storage) => storage.get_crate(name, version),
        }
    }

    fn read_crate(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        match self {
            Storage::Disk(storage) => storage.read_crate(name, version),
            #[cfg(feature = "s3")]
            Storage::S3(storage) => storage.read_crate(name, version),
        }
    }

    fn store_crate(&self, name: &str, version: Version, data: Vec<u8>) -> Result<(), Error> {
        match self {
            Storage::Disk(storage) => storage.store_crate(name, version, data),
            #[cfg(feature = "s3")]
            Storage::S3(storage) => storage.store_crate(name, version, data),
        }
    }

    fn get_readme(&self, name: &str, version: Version) -> Result<String, Error> {
        match self {
            Storage::Disk(storage) => storage.get_readme(name, version),
            #[cfg(feature = "s3")]
            Storage::S3(storage) => storage.get_readme(name, version),
        }
    }

    fn read_readme(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        match self {
            Storage::Disk(storage) => storage.read_readme(name, version),
            #[cfg(feature = "s3")]
            Storage::S3(storage) => storage.read_readme(name, version),
        }
    }

    fn store_readme(&self, name: &str, version: Version, data: String) -> Result<(), Error> {
        match self {
            Storage::Disk(storage) => storage.store_readme(name, version, data),
            #[cfg(feature = "s3")]
            Storage::S3(storage) => storage.store_readme(name, version, data),
        }
    }
}
