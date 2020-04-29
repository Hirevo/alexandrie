mod cli;
#[cfg(feature = "git2")]
mod git2;
use crate::error::Error;
#[cfg(feature = "git2")]
use std::path::Path;
use std::path::PathBuf;

mod r#trait;
use r#trait::Repository as RepositoryTrait;

/// [`LocalRepository`] implements the [`Repository`] trait.
///
/// This git strategy manages a repository on the local machine. It can use *either*
/// a [git-rs](https://docs.rs/git2/0.13.5/git2/) backend, or shell out to
/// the system git CLI (depending on how it is configured).
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

    fn inner(&self) -> &dyn RepositoryTrait {
        match self {
            #[cfg(feature = "git2")]
            Self::Git2(repo) => repo,
            Self::Cli(repo) => repo,
        }
    }
}

// This is an opaque implementation of `trait::Repository`, since
// that trait is private.
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
