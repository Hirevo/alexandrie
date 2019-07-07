use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;

use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{Error, Store};

/// The local on-disk storage strategy.  
///
/// It stores the crates as files in the given directory.  
/// It names the crates as `"{name}-{version}.crate"`.  
/// As there will not be any duplicated names, it doesn't create any subdirectories.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskStorage {
    path: PathBuf,
}

impl DiskStorage {
    /// Create an DiskStorage instance with the specified path.
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<DiskStorage, Error> {
        let path = path.into();
        fs::create_dir_all(&path)?;
        Ok(DiskStorage { path })
    }

    /// Generate a unique filename for the given crate name and version.
    pub fn format_name(name: &str, version: Version) -> String {
        format!("{}-{}.crate", name, version)
    }
}

impl Store for DiskStorage {
    fn get_crate(&self, name: &str, version: Version) -> Result<Vec<u8>, Error> {
        let path = self.path.join(DiskStorage::format_name(name, version));
        let mut file = fs::File::open(&path)?;
        let len = file.metadata()?.len() as usize;
        let mut cursor = io::Cursor::new(Vec::with_capacity(len));
        io::copy(&mut file, &mut cursor)?;
        Ok(cursor.into_inner())
    }

    fn store_crate(&self, name: &str, version: Version, mut data: impl Read) -> Result<(), Error> {
        let path = self.path.join(DiskStorage::format_name(name, version));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        io::copy(&mut data, &mut file)?;
        Ok(())
    }
}
