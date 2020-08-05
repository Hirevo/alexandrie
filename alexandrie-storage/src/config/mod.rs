use serde::{Deserialize, Serialize};

/// The 'disk' configuration.
pub mod disk;

#[cfg(feature = "s3")]
pub mod s3;

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

    /// The S3 storage strategy (crates stored in an S3 bucket).
    #[cfg(feature = "s3")]
    S3(s3::S3StorageConfig),
}

impl From<StorageConfig> for Storage {
    fn from(config: StorageConfig) -> Storage {
        match config {
            StorageConfig::Disk(config) => Storage::Disk(config.into()),
            #[cfg(feature = "s3")]
            StorageConfig::S3(config) => Storage::S3(config.into()),
        }
    }
}
