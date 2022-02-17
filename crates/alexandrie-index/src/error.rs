use thiserror::Error;

/// The Error type for the registry.
///
/// It can represent any kind of error the registry might encounter.
#[derive(Error, Debug)]
pub enum Error {
    /// An IO error (file not found, access forbidden, etc...).
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
    /// JSON (de)serialization error (invalid JSON parsed, etc...).
    #[error("JSON error: {0}")]
    JSONError(#[from] json::Error),
    /// Git2 error.
    #[error("libgit2 error: {0}")]
    #[cfg(feature = "git2")]
    Git2Error(#[from] git2::Error),
    /// Other index-specific error (crate not found, etc...).
    #[error("index-specific error: {0}")]
    IndexError(#[from] IndexError),
}

/// The Error type for Alexandrie's own errors.
#[derive(Error, Debug)]
pub enum IndexError {
    /// The requested crate cannot be found.
    #[error("no crate named '{name}' found")]
    CrateNotFound {
        /// The requested crate's name.
        name: String,
    },
}
