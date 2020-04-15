use crate::error::AlexError;
use crate::index::CrateVersion;
use crate::Error;
use semver::Version;
use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct File<'a> {
    path: &'a Path,
    records: BTreeMap<Version, CrateVersion>,
}

impl<'a> File<'a> {
    pub fn open(path: &'a Path) -> Result<Self, Error> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut records = BTreeMap::new();
        for line in reader.lines() {
            let crate_version: CrateVersion = json::from_str(&line?)?;
            records.insert(crate_version.vers.clone(), crate_version);
        }

        Ok(Self { path, records })
    }

    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        todo!()
    }

    pub fn write(&mut self) -> io::Result<()> {
        let lines: Vec<String> = self
            .records
            .values()
            .map(json::to_string)
            .collect::<Result<Vec<String>, json::Error>>()
            .unwrap(); // we unwrap because any error here is a logic bug in the implementation

        let text = lines.join("\n");

        fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(self.path)?
            .write_all(text.as_bytes())?;

        Ok(())
    }

    pub fn records(&self) -> &BTreeMap<Version, CrateVersion> {
        &self.records
    }

    fn get_mut(&mut self, version: &Version) -> Result<&mut CrateVersion, AlexError> {
        self.records
            .get_mut(version)
            .ok_or(AlexError::CrateNotFound {
                name: "populate me properly".to_string(),
            })
    }

    pub fn yank(&mut self, version: &Version) -> Result<(), Error> {
        self.get_mut(version)?.yank();
        self.write()?;

        Ok(())
    }

    pub fn unyank(&mut self, version: &Version) -> Result<(), Error> {
        self.get_mut(version)?.unyank();
        self.write()?;

        Ok(())
    }
}
