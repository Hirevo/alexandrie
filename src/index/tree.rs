use crate::error::AlexError;
use crate::index::models::CrateVersion;
use crate::Error;
use semver::{Version, VersionReq};
<<<<<<< HEAD
<<<<<<< HEAD
use std::{
    fs, io,
    io::{BufRead, Write},
    path::PathBuf,
};
=======
=======
use std::collections::BTreeMap;
>>>>>>> refactor index::Tree into a Tree + File
use std::fs;
use std::io;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
>>>>>>> add File object

#[derive(Debug, Clone, PartialEq)]
pub struct Tree {
    path: PathBuf,
}

impl Tree {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn compute_record_path(&self, name: &str) -> PathBuf {
        match name.len() {
            1 => self.path.join("1").join(&name),
            2 => self.path.join("2").join(&name),
            3 => self.path.join("3").join(&name[..1]).join(&name),
            _ => self.path.join(&name[0..2]).join(&name[2..4]).join(&name),
        }
    }

    fn file(&self, name: &str) -> io::Result<File> {
        let path = self.compute_record_path(name);
        File::create(path)
    }

    pub fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error> {
        let path = self.compute_record_path(name);
        let file = fs::File::open(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(AlexError::CrateNotFound {
                name: String::from(name),
            }),
            _ => Error::from(err),
        })?;
        let found = io::BufReader::new(file)
            .lines()
            .map(|line| Some(json::from_str(line.ok()?.as_str()).ok()?))
            .flat_map(|ret: Option<CrateVersion>| ret.into_iter())
            .filter(|krate| req.matches(&krate.vers))
            .max_by(|k1, k2| k1.vers.cmp(&k2.vers));
        Ok(found.ok_or_else(|| AlexError::CrateNotFound {
            name: String::from(name),
        })?)
    }

    pub fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        Ok(self.file(name)?.records().values().cloned().collect())
    }

    pub fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        Ok(self.file(name)?.latest_record().unwrap().clone())
    }

    pub fn add_record(&self, record: CrateVersion) -> Result<(), Error> {
        self.file(&record.name)?.add_record(record)
    }

    pub fn yank(&self, name: &str, version: &Version) -> Result<(), Error> {
        self.file(name)?.yank(version)
    }

    pub fn unyank(&self, name: &str, version: &Version) -> Result<(), Error> {
        self.file(name)?.unyank(version)
    }
}

pub struct File {
    path: PathBuf,
    records: BTreeMap<Version, CrateVersion>,
}

impl File {
    pub fn open(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let file = fs::File::open(&path)?;
        Self::from_file(path, file)
    }

    pub fn create(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        fs::create_dir_all(path.parent().unwrap())?;
        let file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&path)?;
        Self::from_file(path, file)
    }

    fn from_file(path: impl Into<PathBuf>, file: fs::File) -> io::Result<Self> {
        let path = path.into();
        let reader = BufReader::new(file);
        let mut records = BTreeMap::new();
        for line in reader.lines() {
            let crate_version: CrateVersion = json::from_str(&line?).expect("malformed json!");
            records.insert(crate_version.vers.clone(), crate_version);
        }

        Ok(Self { path, records })
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
            .open(&self.path)?
            .write_all(text.as_bytes())?;

        Ok(())
    }

    fn crate_name(&self) -> &str {
        self.path.file_name().unwrap().to_str().unwrap()
    }

    pub fn records(&self) -> &BTreeMap<Version, CrateVersion> {
        &self.records
    }

    pub fn latest_record(&self) -> Option<&CrateVersion> {
        self.records.iter().next_back().map(|x| x.1)
    }

    pub fn add_record(&mut self, record: CrateVersion) -> Result<(), Error> {
        if let Some(latest_record) = self.latest_record() {
            if record.vers <= latest_record.vers {
                return Err(AlexError::VersionTooLow {
                    hosted: latest_record.vers.clone(),
                    krate: self.crate_name().to_string(),
                    published: record.vers,
                }
                .into());
            }
        }

        self.records.insert(record.vers.clone(), record).unwrap();
        self.write()?;
        Ok(())
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
