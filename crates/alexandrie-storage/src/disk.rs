use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use semver::Version;

use crate::error::Error;
use crate::Store;

/// The local on-disk storage strategy.  
///
/// It stores the crates as files in the given directory.  
/// It names the crates as `"{name}-{version}.crate"`.  
/// As there will not be any duplicated names, it doesn't create any subdirectories.
#[derive(Debug, Clone, PartialEq)]
pub struct DiskStorage {
    pub(crate) path: PathBuf,
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
        format!("{0}-{1}.crate", name, version)
    }

    /// Generate a unique filename for the html-rendered readme page for the given crate name and version.
    pub fn format_readme_name(name: &str, version: Version) -> String {
        format!("{0}-{1}.readme", name, version)
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

    fn read_crate(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        let path = self.path.join(DiskStorage::format_name(name, version));
        let file = fs::File::open(&path)?;
        Ok(Box::new(file))
    }

    fn store_crate(&self, name: &str, version: Version, data: Vec<u8>) -> Result<(), Error> {
        let path = self.path.join(DiskStorage::format_name(name, version));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(&data)?;
        Ok(())
    }

    fn get_readme(&self, name: &str, version: Version) -> Result<String, Error> {
        let path = self
            .path
            .join(DiskStorage::format_readme_name(name, version));
        Ok(fs::read_to_string(path)?)
    }

    fn read_readme(&self, name: &str, version: Version) -> Result<Box<dyn Read>, Error> {
        let path = self
            .path
            .join(DiskStorage::format_readme_name(name, version));
        let file = fs::File::open(&path)?;
        Ok(Box::new(file))
    }

    fn store_readme(&self, name: &str, version: Version, data: String) -> Result<(), Error> {
        let path = self
            .path
            .join(DiskStorage::format_readme_name(name, version));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;
        file.write_all(data.as_bytes())?;
        Ok(())
    }
}
