use std::path::PathBuf;

use serde::{Serialize, Deserialize};
use semver::{Version, VersionReq};

mod cli;
pub use cli::*;

use crate::{Error, Crate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Index {
    CLIIndex(CLIIndex),
}

pub trait Indexer {
    fn url(&self) -> Result<String, Error>;
    fn refresh(&self) -> Result<(), Error>;
    fn max_version(&self, name: &str) -> Result<Version, Error>;
    fn index_crate(&self, name: &str) -> PathBuf;
    fn match_crate(&self, name: &str, req: VersionReq) -> Result<Crate, Error>;
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

    fn max_version(&self, name: &str) -> Result<Version, Error> {
        match self {
            Index::CLIIndex(idx) => idx.max_version(name),
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
