use std::fs;
use std::io::{self, BufRead};
use std::path::PathBuf;
use std::process::{Command, Stdio};

use semver::{Version, VersionReq};
use serde::{Serialize, Deserialize};

use crate::{AlexError, Crate, Error, Indexer};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CLIIndex {
    path: PathBuf,
}

impl CLIIndex {
    pub fn new<P: Into<PathBuf>>(dir: P) -> Result<CLIIndex, Error> {
        let path = dir.into();

        Ok(CLIIndex { path })
    }
}

impl Indexer for CLIIndex {
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

    fn index_crate(&self, name: &str) -> PathBuf {
        match name.len() {
            1 => self.path.join("1").join(&name),
            2 => self.path.join("2").join(&name),
            3 => self.path.join("3").join(&name[..1]).join(&name),
            _ => self.path.join(&name[0..2]).join(&name[2..4]).join(&name),
        }
    }

    fn match_crate(&self, name: &str, req: VersionReq) -> Result<Crate, Error> {
        let path = self.index_crate(name);
        let file = fs::File::open(path).map_err(|err| match err.kind() {
            io::ErrorKind::NotFound => Error::from(AlexError::CrateNotFound(String::from(name))),
            _ => Error::from(err),
        })?;
        let found = io::BufReader::new(file)
            .lines()
            .map(|line| Some(json::from_str(line.ok()?.as_str()).ok()?))
            .flat_map(|ret: Option<Crate>| ret.into_iter())
            .filter(|krate| req.matches(&krate.vers))
            .max_by(|k1, k2| k1.vers.cmp(&k2.vers));
        Ok(found.ok_or_else(|| AlexError::CrateNotFound(String::from(name)))?)
    }

    fn max_version(&self, name: &str) -> Result<Version, Error> {
        let path = self.index_crate(name);
        let reader = io::BufReader::new(fs::File::open(path)?);
        Ok(reader
            .lines()
            .map(|line| Ok(json::from_str::<Crate>(line?.as_str())?.vers))
            .collect::<Result<Vec<Version>, Error>>()?
            .into_iter()
            .max()
            .expect("at least one version to exist"))
    }
}
