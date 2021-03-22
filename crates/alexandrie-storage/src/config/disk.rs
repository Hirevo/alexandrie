use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::disk::DiskStorage;

/// The configuration struct for the 'disk' storage strategy.
///
/// ```toml
/// [storage]
/// type = "disk"          # required
/// path = "crate-storage" # required
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DiskStorageConfig {
    /// The path to a local directory in which to store crate blobs.
    pub path: PathBuf,
}

impl From<DiskStorageConfig> for DiskStorage {
    fn from(config: DiskStorageConfig) -> DiskStorage {
        DiskStorage { path: config.path }
    }
}
