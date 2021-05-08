use std::convert::TryInto;
use std::io;

use diesel::result::Error as SQLError;
use hex::FromHexError as HexError;
use io::Error as IOError;
use json::Error as JSONError;
use semver::{ReqParseError, SemVerError as SemverError, Version};
use thiserror::Error;
use toml::de::Error as TOMLError;

use alexandrie_index::error::Error as IndexError;
use alexandrie_storage::error::Error as StorageError;

use crate::db::models::Author;

/// The Error type for the registry.
///
/// It can represent any kind of error the registry might encounter.
#[derive(Error, Debug)]
pub enum Error {
    /// An IO error (file not found, access forbidden, etc...).
    #[error("IO error: {0}")]
    IOError(#[from] IOError),
    /// JSON (de)serialization error (invalid JSON parsed, etc...).
    #[error("JSON error: {0}")]
    JSONError(#[from] JSONError),
    /// TOML (de)serialization error (invalid TOML parsed, etc...).
    #[error("TOML error: {0}")]
    TOMLError(#[from] TOMLError),
    /// SQL error (invalid queries, database disconnections, etc...).
    #[error("SQL error: {0}")]
    SQLError(#[from] SQLError),
    /// Version parsing errors (invalid version format parsed, etc...).
    #[error("semver error: {0}")]
    SemverError(#[from] SemverError),
    /// Version requirement parsing errors (invalid version requirement format parsed, etc...).
    #[error("semver req parse error: {0}")]
    SemverReqParseError(#[from] ReqParseError),
    /// Hexadecimal decoding errors (odd length, etc...).
    #[error("hex error: {0}")]
    HexError(#[from] HexError),
    /// Alexandrie's custom errors (crate not found, invalid token, etc...).
    #[error("alexandrie error: {0}")]
    AlexError(#[from] AlexError),
    /// Index-specific errors.
    #[error("{0}")]
    IndexError(#[from] IndexError),
    /// Storage-specific errors.
    #[error("{0}")]
    StorageError(#[from] StorageError),
}

/// The Error type for Alexandrie's own errors.
#[derive(Error, Debug)]
pub enum AlexError {
    /// The requested crate cannot be found.
    #[error("no crate named '{name}' found")]
    CrateNotFound {
        /// The requested crate's name.
        name: String,
    },
    /// The crate is not owned by the user.
    #[error("you are not an owner of '{name}'")]
    CrateNotOwned {
        /// The involved crate's name.
        name: String,
        /// The involved author.
        author: Author,
    },
    /// The published crate version is lower than the current hosted version.
    #[error("the published version is too low (hosted version is {hosted}, and thus {published} <= {hosted})")]
    VersionTooLow {
        /// The krate's name.
        krate: String,
        /// The available hosted version.
        hosted: Version,
        /// The proposed version to be published.
        published: Version,
    },
    /// The token used to access the registry is invalid.
    #[error("invalid token")]
    InvalidToken,
    /// The request is invalid because of a required query parameter.
    #[error("missing query parameters: {missing_params:?}")]
    MissingQueryParams {
        /// The list of missing query parameters.
        missing_params: &'static [&'static str],
    },
}

// impl IntoResponse for Error {
//     fn into_response(self) -> Response {
//         error!("constructing error response: {0}", self);
//         let message = match self {
//             Error::AlexError(err) => err.to_string(),
//             _ => "internal server error".to_string(),
//         };

//         utils::response::error(http::StatusCode::InternalServerError, message)
//     }
// }

impl TryInto<IOError> for Error {
    type Error = ();

    fn try_into(self) -> Result<IOError, Self::Error> {
        match self {
            Error::IOError(err) => Ok(err),
            _ => Err(()),
        }
    }
}

impl TryInto<JSONError> for Error {
    type Error = ();

    fn try_into(self) -> Result<JSONError, Self::Error> {
        match self {
            Error::JSONError(err) => Ok(err),
            _ => Err(()),
        }
    }
}

impl TryInto<TOMLError> for Error {
    type Error = ();

    fn try_into(self) -> Result<TOMLError, Self::Error> {
        match self {
            Error::TOMLError(err) => Ok(err),
            _ => Err(()),
        }
    }
}

impl TryInto<SQLError> for Error {
    type Error = ();

    fn try_into(self) -> Result<SQLError, Self::Error> {
        match self {
            Error::SQLError(err) => Ok(err),
            _ => Err(()),
        }
    }
}

impl TryInto<SemverError> for Error {
    type Error = ();

    fn try_into(self) -> Result<SemverError, Self::Error> {
        match self {
            Error::SemverError(err) => Ok(err),
            _ => Err(()),
        }
    }
}

impl TryInto<AlexError> for Error {
    type Error = ();

    fn try_into(self) -> Result<AlexError, Self::Error> {
        match self {
            Error::AlexError(err) => Ok(err),
            _ => Err(()),
        }
    }
}
