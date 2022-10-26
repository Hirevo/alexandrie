use semver::{Version, VersionReq};

pub mod config;
pub mod error;

mod index;
mod models;
mod tree;

pub use index::*;
pub use models::{CrateDependency, CrateDependencyKind, CrateVersion};

use crate::cli::CommandLineIndex;
use crate::error::Error;

#[cfg(feature = "git2")]
use crate::git2::Git2Index;

/// The crate indexing management strategy type.
///
/// It represents which index management strategy is currently used.
pub enum Index {
    /// Manages the crate index through the invocation of the "git" shell command.
    CommandLine(CommandLineIndex),
    /// Manages the crate index using [**`libgit2`**].
    /// [libgit2]: https://libgit2.org
    #[cfg(feature = "git2")]
    Git2(Git2Index),
}

/// The required trait that any crate index management type must implement.
pub trait Indexer {
    /// Gives back the URL of the managed crate index.
    fn url(&self) -> Result<String, Error>;
    /// Refreshes the managed crate index (in case another instance made modification to it).
    fn refresh(&self) -> Result<(), Error>;
    /// Retrieves all the version records of a crate.
    fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error>;
    /// Retrieves the latest version record of a crate.
    fn latest_record(&self, name: &str) -> Result<CrateVersion, Error>;
    /// Retrieves the latest crate version record that matches the given name and version requirement.
    fn match_record(&self, name: &str, req: VersionReq) -> Result<Option<CrateVersion>, Error>;
    /// Commits and pushes changes upstream.
    fn commit_and_push(&self, msg: &str) -> Result<(), Error>;
    /// Adds a new crate record into the index.
    fn add_record(&self, record: CrateVersion) -> Result<(), Error>;
    /// Alters an index's crate version record with the passed-in function.
    fn alter_record<F>(&self, name: &str, version: Version, func: F) -> Result<(), Error>
    where
        F: FnOnce(&mut CrateVersion);
    /// Yanks a crate version.
    fn yank_record(&self, name: &str, version: Version) -> Result<(), Error> {
        self.alter_record(name, version, |krate| krate.yanked = Some(true))
    }
    /// Un-yanks a crate version.
    fn unyank_record(&self, name: &str, version: Version) -> Result<(), Error> {
        self.alter_record(name, version, |krate| krate.yanked = Some(false))
    }
}

impl Indexer for Index {
    fn url(&self) -> Result<String, Error> {
        match self {
            Index::CommandLine(idx) => idx.url(),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.url(),
        }
    }

    fn refresh(&self) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.refresh(),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.refresh(),
        }
    }

    fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        match self {
            Index::CommandLine(idx) => idx.all_records(name),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.all_records(name),
        }
    }

    fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        match self {
            Index::CommandLine(idx) => idx.latest_record(name),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.latest_record(name),
        }
    }

    fn match_record(&self, name: &str, req: VersionReq) -> Result<Option<CrateVersion>, Error> {
        match self {
            Index::CommandLine(idx) => idx.match_record(name, req),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.match_record(name, req),
        }
    }

    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.commit_and_push(msg),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.commit_and_push(msg),
        }
    }

    fn add_record(&self, record: CrateVersion) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.add_record(record),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.add_record(record),
        }
    }

    fn alter_record<F>(&self, name: &str, version: Version, func: F) -> Result<(), Error>
    where
        F: FnOnce(&mut CrateVersion),
    {
        match self {
            Index::CommandLine(idx) => idx.alter_record(name, version, func),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.alter_record(name, version, func),
        }
    }

    fn yank_record(&self, name: &str, version: Version) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.yank_record(name, version),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.yank_record(name, version),
        }
    }

    fn unyank_record(&self, name: &str, version: Version) -> Result<(), Error> {
        match self {
            Index::CommandLine(idx) => idx.unyank_record(name, version),
            #[cfg(feature = "git2")]
            Index::Git2(idx) => idx.unyank_record(name, version),
        }
    }
}
