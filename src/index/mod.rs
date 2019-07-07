use std::path::PathBuf;

use semver::VersionReq;
use serde::{Deserialize, Serialize};

mod cli;
pub use cli::*;

use crate::{Crate, Error};

/// The crate indexing management strategy type.
///
/// It represents which index management strategy is currently used.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Index {
    /// Manages the crate index through the invocation of the "git" shell command.
    #[serde(rename = "cli")]
    CLIIndex(CLIIndex),
}

/// The required trait that any crate index management type must implement.
pub trait Indexer {
    /// Gives back the URL of the managed crate index.
    fn url(&self) -> Result<String, Error>;
    /// Refreshes the managed crate index (in case another instance made modification to it).
    fn refresh(&self) -> Result<(), Error>;
    /// Retrives the latest version of a crate.
    fn latest_crate(&self, name: &str) -> Result<Crate, Error>;
    /// Retrives the filepath to the saved crate metadata.
    fn index_crate(&self, name: &str) -> PathBuf;
    /// Retrives the crate metadata for the given name and version.
    fn match_crate(&self, name: &str, req: VersionReq) -> Result<Crate, Error>;
    /// Commit and push changes upstream.
    fn commit_and_push(&self, msg: &str) -> Result<(), Error>;
}

impl Indexer for Index {
    fn url(&self) -> Result<String, Error> {
        match self {
            Index::CLIIndex(idx) => idx.url(),
        }
    }

    fn refresh(&self) -> Result<(), Error> {
        match self {
            Index::CLIIndex(idx) => idx.refresh(),
        }
    }

    fn latest_crate(&self, name: &str) -> Result<Crate, Error> {
        match self {
            Index::CLIIndex(idx) => idx.latest_crate(name),
        }
    }

    fn index_crate(&self, name: &str) -> PathBuf {
        match self {
            Index::CLIIndex(idx) => idx.index_crate(name),
        }
    }

    fn match_crate(&self, name: &str, req: VersionReq) -> Result<Crate, Error> {
        match self {
            Index::CLIIndex(idx) => idx.match_crate(name, req),
        }
    }

    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        match self {
            Index::CLIIndex(idx) => idx.commit_and_push(msg),
        }
    }
}
