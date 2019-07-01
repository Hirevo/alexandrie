use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use crate::{AlexError, Crate, Error};

#[derive(Debug, Clone, PartialEq)]
pub struct Index {
    path: PathBuf,
}

impl Index {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Result<Index, Error> {
        let path = dir.into();

        Ok(Index { path })
    }

    pub fn refresh(&self) -> Result<(), Error> {
        Command::new("git")
            .arg("pull")
            .arg("--ff-only")
            .current_dir(self.path.canonicalize()?)
            .spawn()?
            .wait()?;

        Ok(())
    }

    pub fn url(&self) -> Result<String, Error> {
        let output = Command::new("git")
            .arg("remote")
            .arg("get-url")
            .arg("origin")
            .stdout(Stdio::piped())
            .current_dir(self.path.canonicalize()?)
            .output()?;

        Ok(String::from_utf8_lossy(output.stdout.as_slice()).into())
    }

    pub fn find_crate(&self, name: &str) -> Result<Crate, Error> {
        let path = match name.len() {
            1 => self.path.join("1").join(&name),
            2 => self.path.join("2").join(&name),
            3 => self.path.join("3").join(&name[..1]).join(&name),
            _ => self.path.join(&name[0..2]).join(&name[2..4]).join(&name),
        };
        let file = fs::File::open(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(AlexError::CrateNotFound(String::from(name))),
            _ => Error::from(err),
        })?;
        Ok(json::from_reader(file)?)
    }
}
