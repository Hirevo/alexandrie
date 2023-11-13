use std::path::PathBuf;
use std::process::{Command, Stdio};

use semver::{Version, VersionReq};

use crate::error::Error;
use crate::tree::Tree;
use crate::{ConfigFile, CrateVersion, Indexer};

/// The 'command-line' crate index management strategy type.
///
/// It manages the crate index through the invocation of "git" shell commands.
#[derive(Debug, Clone, PartialEq)]
pub struct CommandLineIndex {
    repo: Repository,
    tree: Tree,
}

impl CommandLineIndex {
    /// Create a CommandLineIndex instance with the given path.
    pub fn new<P: Into<PathBuf>>(path: P) -> CommandLineIndex {
        let path = path.into();
        let repo = Repository { path: path.clone() };
        let tree = Tree::new(path);
        CommandLineIndex { repo, tree }
    }
}

impl Indexer for CommandLineIndex {
    fn url(&self) -> Result<String, Error> {
        self.repo.url()
    }

    fn refresh(&self) -> Result<(), Error> {
        self.repo.refresh()
    }

    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        self.repo.commit_and_push(msg)
    }

    fn configuration(&self) -> Result<ConfigFile, Error> {
        self.tree.configuration()
    }

    fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error> {
        self.tree.match_record(name, req)
    }

    fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        self.tree.all_records(name)
    }

    fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        self.tree.latest_record(name)
    }

    fn add_record(&self, record: CrateVersion) -> Result<(), Error> {
        self.tree.add_record(record)
    }

    fn alter_record<F>(&self, name: &str, version: Version, func: F) -> Result<(), Error>
    where
        F: FnOnce(&mut CrateVersion),
    {
        self.tree.alter_record(name, version, func)
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Repository {
    path: PathBuf,
}

impl Repository {
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
