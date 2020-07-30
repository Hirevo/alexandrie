use thiserror::Error;

/// The Error type for the registry.
///
/// It can represent any kind of error the registry might encounter.
#[derive(Error, Debug)]
pub enum Error {
    /// An IO error (file not found, access forbidden, etc...).
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),

    /// An S3 `GET` request failed.
    #[cfg(feature = "s3")]
    #[error("S3 GET error: {0}")]
    S3GetError(#[from] rusoto_core::RusotoError<rusoto_s3::GetObjectError>),

    /// An S3 `PUT` request failed.
    #[cfg(feature = "s3")]
    #[error("S3 PUT error: {0}")]
    S3PutError(#[from] rusoto_core::RusotoError<rusoto_s3::PutObjectError>),
}
