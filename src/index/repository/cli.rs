use std::path::PathBuf;
use std::process::{Command, Stdio};

use super::Repo;
use crate::error::Error;

/// The 'command-line' crate index management strategy type.
///
/// It manages the crate index through the invocation of "git" shell commands.
#[derive(Debug, Clone, PartialEq)]
pub struct Repository {
    path: PathBuf,
}

impl Repository {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { path: path.into() }
    }
}

impl Repo for Repository {
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
}
