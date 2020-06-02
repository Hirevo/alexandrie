use semver::{Version, VersionReq};
use std::convert::TryFrom;
use std::path::PathBuf;

mod models;
mod tree;
use tree::Tree;
mod repository;
use repository::Repository;
use serde::{Deserialize, Serialize};

pub use models::{CrateDependency, CrateDependencyKind, CrateVersion};

use crate::error::Error;

pub struct Index {
    repo: Box<dyn Repository + Send + Sync>,
    tree: Tree,
}

impl Index {
    pub fn with_repo(path: PathBuf, repo: impl Repository + Send + Sync + 'static) -> Self {
        let repo = Box::new(repo);
        let tree = Tree::new(path);
        Self { repo, tree }
    }
}

/// The configuration struct for the 'git2' index management strategy.
///
/// ```toml
/// [index]
/// type = "git2" | "command-line"  # required
/// path = "crate-index"            # required
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
#[serde(rename_all = "kebab-case")]
pub enum Config {
    Git2 { path: PathBuf },
    CommandLine { path: PathBuf },
}

impl TryFrom<Config> for Index {
    type Error = Error;
    fn try_from(config: Config) -> Result<Self, Self::Error> {
        match config {
            Config::CommandLine { path } => {
                let repo = repository::cli::Repo::new(path.clone());
                Ok(Self::with_repo(path, repo))
            }
            Config::Git2 { path } => {
                let repo = repository::git2::Repo::new(&path)?;
                Ok(Self::with_repo(path, repo))
            }
        }
    }
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
    fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error>;
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
        self.repo.url()
    }
    fn refresh(&self) -> Result<(), Error> {
        self.repo.refresh()
    }
    fn all_records(&self, name: &str) -> Result<Vec<CrateVersion>, Error> {
        self.tree.all_records(name)
    }
    fn latest_record(&self, name: &str) -> Result<CrateVersion, Error> {
        self.tree.latest_record(name)
    }
    fn match_record(&self, name: &str, req: VersionReq) -> Result<CrateVersion, Error> {
        self.tree.match_record(name, req)
    }
    fn commit_and_push(&self, msg: &str) -> Result<(), Error> {
        self.repo.commit_and_push(msg)
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

#[cfg(test)]
mod tests {
    use super::Config;
    #[test]
    fn from_config() {
        match toml::from_str(
            r#"
        type = "git2"
        path = "crate-index"
        "#,
        )
        .unwrap()
        {
            Config::Git2 { .. } => (),
            Config::CommandLine { .. } => panic!("deserialization failed!"),
        }

        match toml::from_str(
            r#"
        type = "command-line"
        path = "crate-index"
        "#,
        )
        .unwrap()
        {
            Config::Git2 { .. } => panic!("deserialization failed!"),
            Config::CommandLine { .. } => (),
        }
    }
}
