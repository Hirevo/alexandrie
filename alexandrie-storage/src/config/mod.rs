use serde::{Deserialize, Serialize};

/// The 'disk' configuration.
pub mod disk;

use crate::config::disk::DiskStorageConfig;
use crate::Storage;

/// The configuration enum for storage strategies.
///
/// ```toml
/// [storage]
/// type = "<...>" # required, replace "<...>" by the selected strategy.
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum StorageConfig {
    /// The 'disk' storage strategy (local on-disk storage).
    Disk(DiskStorageConfig),
}

impl From<StorageConfig> for Storage {
    fn from(config: StorageConfig) -> Storage {
        match config {
            StorageConfig::Disk(config) => Storage::Disk(config.into()),
        }
    }
}
