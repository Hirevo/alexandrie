use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use semver::{Version, VersionReq};

use crate::error::{AlexError, Error};
use crate::index::{CrateVersion, Indexer};

/// The 'command-line' crate index management strategy type.
///
/// It manages the crate index through the invocation of "git" shell commands.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandLineIndex {
    /// The path of the crate index.
    pub(crate) path: PathBuf,
}

impl CommandLineIndex {
    /// Create a CommandLineIndex instance with the given path.
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<CommandLineIndex, Error> {
        let path = path.into();
        Ok(CommandLineIndex { path })
    }

    fn compute_record_path(&self, name: &str) -> PathBuf {
        match name.len() {
            1 => self.path.join("1").join(&name),
            2 => self.path.join("2").join(&name),
            3 => self.path.join("3").join(&name[..1]).join(&name),
            _ => self.path.join(&name[0..2]).join(&name[2..4]).join(&name),
        }
    }
}

impl Indexer for CommandLineIndex {
    fn url(&self) -> Result<String, Error> {
        let output = Command::new("git")
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .stdout(Stdio::piped())
            .current_dir(self.path.canonicalize()?)
            .output()?;

        Ok(String::from_utf8_lossy(output.stdout.as_slice()).into())
    }

    fn refresh(&self) -> Result<(), Error> {
        Command::new("git")
            .arg("pull")
            .arg("--ff-only")
            .current_dir(self.path.canonicalize()?)
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        Command::new("git")
            .arg("add")
            .arg("--all")
            .current_dir(&self.path)
            .spawn()?
            .wait()?;
        Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(msg)
            .current_dir(&self.path)
            .spawn()?
            .wait()?;
        Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("master")
            .current_dir(&self.path)
            .spawn()?
            .wait()?;

        Ok(())
    }

    fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error> {
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

    fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        let path = self.compute_record_path(name);
        let reader = io::BufReader::new(fs::File::open(path)?);
        reader
            .lines()
            .map(|line| Ok(json::from_str::<CrateVersion>(line?.as_str())?))
            .collect()
    }

    fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        let records = self.all_records(name)?;
        Ok(records
            .into_iter()
            .max_by(|k1, k2| k1.vers.cmp(&k2.vers))
            .expect("at least one version should exist"))
    }

    fn add_record(&self, record: CrateVersion) -> Result<(), Error> {
        let path = self.compute_record_path(record.name.as_str());

        if let Ok(file) = fs::File::open(&path) {
            let reader = io::BufReader::new(file);
            let records = reader
                .lines()
                .map(|line| Ok(json::from_str::<CrateVersion>(line?.as_str())?))
                .collect::<Result<Vec<CrateVersion>, Error>>()?;
            let latest = records
                .into_iter()
                .max_by(|k1, k2| k1.vers.cmp(&k2.vers))
                .expect("at least one record should exist");
            if record.vers <= latest.vers {
                return Err(Error::from(AlexError::VersionTooLow {
                    krate: record.name,
                    hosted: latest.vers,
                    published: record.vers,
                }));
            }
        } else {
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

    fn alter_record<F>(&self, name: &str, version: Version, func: F) -> Result<(), Error>
    where
        F: FnOnce(&mut CrateVersion),
    {
        let path = self.compute_record_path(name);
        let file = fs::File::open(path.as_path()).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(AlexError::CrateNotFound {
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
                Error::from(AlexError::CrateNotFound {
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
