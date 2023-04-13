use std::fs;
use std::io;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use semver::{Version, VersionReq};

use crate::error::IndexError;
use crate::models::CrateVersion;
use crate::Error;

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

    pub fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error> {
        let path = self.compute_record_path(name);
        let file = fs::File::open(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(IndexError::CrateNotFound {
                name: String::from(name),
            }),
            _ => Error::from(err),
        })?;

        let found = io::BufReader::new(file)
            .lines()
            .filter_map(|line| json::from_str::<CrateVersion>(line.ok()?.as_str()).ok())
            .filter(|krate| req.matches(&krate.vers))
            .max_by(|k1, k2| k1.vers.cmp(&k2.vers));

        Ok(found.ok_or_else(|| IndexError::CrateNotFound {
            name: String::from(name),
        })?)
    }

    pub fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        let path = self.compute_record_path(name);
        let reader = io::BufReader::new(fs::File::open(path)?);
        reader
            .lines()
            .map(|line| Ok(json::from_str::<CrateVersion>(line?.as_str())?))
            .collect()
    }

    pub fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        let records = self.all_records(name)?;
        Ok(records
            .into_iter()
            .max_by(|k1, k2| k1.vers.cmp(&k2.vers))
            .expect("at least one version should exist"))
    }

    pub fn add_record(&self, record: CrateVersion) -> Result<(), Error> {
        let path = self.compute_record_path(record.name.as_str());

        if !path.exists() {
            let parent = path.parent().unwrap();
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(path)?;
        json::to_writer(&mut file, &record)?;
        writeln!(file)?;
        file.flush()?;

        Ok(())
    }

    pub fn alter_record<F>(&self, name: &str, version: Version, func: F) -> Result<(), Error>
    where
        F: FnOnce(&mut CrateVersion),
    {
        let path = self.compute_record_path(name);
        let file = fs::File::open(path.as_path()).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(IndexError::CrateNotFound {
                name: String::from(name),
            }),
            _ => Error::from(err),
        })?;
        let mut krates: Vec<CrateVersion> = {
            let mut out = Vec::new();
            for line in io::BufReader::new(file).lines() {
                let krate = json::from_str(line?.as_str())?;
                out.push(krate);
            }
            out
        };
        let found = krates
            .iter_mut()
            .find(|krate| krate.vers == version)
            .ok_or_else(|| {
                Error::from(IndexError::CrateNotFound {
                    name: String::from(name),
                })
            })?;

        func(found);

        let lines = krates
            .into_iter()
            .map(|krate| json::to_string(&krate))
            .collect::<Result<Vec<String>, _>>()?;
        fs::write(path.as_path(), lines.join("\n") + "\n")?;

        Ok(())
    }
}
