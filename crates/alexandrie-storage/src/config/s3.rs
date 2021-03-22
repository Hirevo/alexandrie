use crate::s3::S3Storage;
use rusoto_core::Region;
use serde::{Deserialize, Serialize};

/// The configuration struct for the 's3' storage strategy.
///
/// ```toml
/// [storage]
/// type = "s3"             # required
/// region = [ "us-west-1"] # required
/// # region = [ "custom", "http://localhost:9000" ] # custom regions; e.g., local testing with minio
/// bucket = "bucket-name"  # required
/// key_prefix = "path/inside/bucket" # optional; defaults to "crates"
/// ```
///
/// AWS credentials can be provided by any of the methods supported by
/// [`rusoto_credential::DefaultCredentialsProvider`](https://docs.rs/rusoto_credential/0.44.0/rusoto_credential/struct.DefaultCredentialsProvider.html).
/// The first source it looks for is the pair of environment variables
/// `AWS_ACCESS_KEY_ID` and `AWS_SECRET_ACCESS_KEY`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct S3StorageConfig {
    /// The S3 region.
    pub region: Region,
    /// The S3 bucket name.
    pub bucket: String,
    /// The prefix to use for storage inside the bucket. Defaults to `crates`.
    /// Should not end with a `/`.
    #[serde(default = "default_key_prefix")]
    pub key_prefix: String,
}

fn default_key_prefix() -> String {
    "crates".to_string()
}

impl From<S3StorageConfig> for S3Storage {
    fn from(config: S3StorageConfig) -> Self {
        Self::new(config.region, config.bucket, config.key_prefix)
    }
}
