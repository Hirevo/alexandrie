use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::index::git2::Git2Index;

/// The configuration struct for the 'git2' index management strategy.
///
/// ```toml
/// [index]
/// type = "git2"        # required
/// path = "crate-index" # required
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Git2IndexConfig {
    /// The path to the local index repository.
    pub path: PathBuf,
}

impl From<Git2IndexConfig> for Git2Index {
    fn from(config: Git2IndexConfig) -> Git2Index {
        Git2Index::new(config.path).expect("could not initialize the 'git2' index")
    }
}
