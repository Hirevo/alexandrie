use std::path::PathBuf;

use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};

/// Index management through `git` shell command invocations.
pub mod cli;

use crate::error::Error;
use crate::index::cli::CommandLineIndex;
use crate::krate::Crate;

/// The crate indexing management strategy type.
///
/// It represents which index management strategy is currently used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Index {
    /// Manages the crate index through the invocation of the "git" shell command.
    #[serde(rename = "command-line")]
    CommandLine(CommandLineIndex),
    // TODO: Add an `Indexer` implementation using `git2`.
    // Git2(Git2Index),
}

/// The required trait that any crate index management type must implement.
pub trait Indexer {
    /// Gives back the URL of the managed crate index.
    fn url(&self) -> Result<String, Error>;
    /// Refreshes the managed crate index (in case another instance made modification to it).
    fn refresh(&self) -> Result<(), Error>;
    /// Retrieves the latest version of a crate.
    fn latest_crate(&self, name: &str) -> Result<Crate, Error>;
    /// Retrieves the filepath to the saved crate metadata.
    fn index_crate(&self, name: &str) -> PathBuf;
    /// Retrieves the crate metadata for the given name and version.
    fn match_crate(&self, name: &str, req: VersionReq) -> Result<Crate, Error>;
    /// Commits and pushes changes upstream.
    fn commit_and_push(&self, msg: &str) -> Result<(), Error>;
    /// Alters the index's crate record with the passed-in function.
    fn alter_crate(
        &self,
        name: &str,
        version: Version,
        func: impl FnOnce(&mut Crate),
    ) -> Result<(), Error>;
    /// Yanks a crate version.
    fn yank_crate(&self, name: &str, version: Version) -> Result<(), Error> {
        self.alter_crate(name, version, |krate| krate.yanked = Some(true))
    }
    /// Un-yanks a crate version.
    fn unyank_crate(&self, name: &str, version: Version) -> Result<(), Error> {
        self.alter_crate(name, version, |krate| krate.yanked = Some(false))
    }
}

impl Indexer for Index {
    fn url(&self) -> Result<String, Error> {
        match self {
            Index::CommandLine(idx) => idx.url(),
        }
    }

    fn refresh(&self) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.refresh(),
        }
    }

    fn latest_crate(&self, name: &str) -> Result<Crate, Error> {
        match self {
            Index::CommandLine(idx) => idx.latest_crate(name),
        }
    }

    fn index_crate(&self, name: &str) -> PathBuf {
        match self {
            Index::CommandLine(idx) => idx.index_crate(name),
        }
    }

    fn match_crate(&self, name: &str, req: VersionReq) -> Result<Crate, Error> {
        match self {
            Index::CommandLine(idx) => idx.match_crate(name, req),
        }
    }

    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.commit_and_push(msg),
        }
    }

    fn alter_crate(
        &self,
        name: &str,
        version: Version,
        func: impl FnOnce(&mut Crate),
    ) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.alter_crate(name, version, func),
        }
    }

    fn yank_crate(&self, name: &str, version: Version) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.yank_crate(name, version),
        }
    }

    fn unyank_crate(&self, name: &str, version: Version) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.unyank_crate(name, version),
        }
    }
}
