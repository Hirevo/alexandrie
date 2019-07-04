use std::path::PathBuf;
use std::io::{self, Read};
use std::fs;

use serde::{Serialize, Deserialize};
use semver::Version;

use crate::{Store, Error};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskStorage {
    path: PathBuf,
}

impl DiskStorage {
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<DiskStorage, Error> {
        Ok(DiskStorage { path: path.into() })
    }

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
        let mut file = fs::OpenOptions::new().create_new(true).write(true).open(&path)?;
        io::copy(&mut data, &mut file)?;
        Ok(())
    }
}
