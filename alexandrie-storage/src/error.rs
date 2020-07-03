use thiserror::Error;

/// The Error type for the registry.
///
/// It can represent any kind of error the registry might encounter.
#[derive(Error, Debug)]
pub enum Error {
    /// An IO error (file not found, access forbidden, etc...).
    #[error("IO error: {0}")]
    IOError(#[from] std::io::Error),
}
