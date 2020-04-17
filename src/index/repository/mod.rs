mod cli;
#[cfg(feature = "git2")]
mod git2;
use crate::error::Error;
#[cfg(feature = "git2")]
use std::path::Path;
use std::path::PathBuf;

trait Repo {
    fn url(&self) -> Result<String, Error>;

    fn refresh(&self) -> Result<(), Error>;

    fn commit_and_push(&self, msg: &str) -> Result<(), Error>;
}

pub enum Repository {
    #[cfg(feature = "git2")]
    Git2(git2::Repository),
    Cli(cli::Repository),
}

impl Repository {
    #[cfg(feature = "git2")]
    pub fn new_git2<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        Ok(Self::Git2(git2::Repository::new(path)?))
    }

    pub fn new_cli<P: Into<PathBuf>>(path: P) -> Self {
        Self::Cli(cli::Repository::new(path))
    }

    fn inner(&self) -> &dyn Repo {
        match self {
            #[cfg(feature = "git2")]
            Self::Git2(repo) => repo,
            Self::Cli(repo) => repo,
        }
    }
}

impl Repository {
    pub fn url(&self) -> Result<String, Error> {
        self.inner().url()
    }
    pub fn refresh(&self) -> Result<(), Error> {
        self.inner().refresh()
    }
    pub fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        self.inner().commit_and_push(msg)
    }
}
