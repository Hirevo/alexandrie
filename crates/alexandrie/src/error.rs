use std::convert::TryInto;
use std::fmt::{Debug, Display};
use std::io;

use axum::response::{IntoResponse, Response};
use axum::Json;
use diesel::result::Error as SQLError;
use hex::FromHexError as HexError;
use io::Error as IOError;
use json::Error as JSONError;
use semver::{Error as SemverError, Version};
use tantivy::directory::error::OpenDirectoryError;
use tantivy::TantivyError;
use thiserror::Error;
use toml::de::Error as TOMLError;

use alexandrie_index::error::Error as IndexError;
use alexandrie_storage::error::Error as StorageError;

use crate::db::models::Author;

/// Represents an error from the programmatic API.
pub struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        // Transform endpoint errors into the format expected by Cargo.
        Json(json::json!({
            "errors": [{
                "detail": self.0.to_string(),
            }]
        }))
        .into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl ApiError {
    /// Constructs an instance from a single message.
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Self(anyhow::Error::msg(message))
    }
}

#[cfg(feature = "frontend")]
/// Represents an error from the frontend.
pub struct FrontendError(anyhow::Error);

#[cfg(feature = "frontend")]
impl IntoResponse for FrontendError {
    fn into_response(self) -> Response {
        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

#[cfg(feature = "frontend")]
impl<E> From<E> for FrontendError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

#[cfg(feature = "frontend")]
impl FrontendError {
    /// Constructs an instance from a single message.
    pub fn msg<M>(message: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Self(anyhow::Error::msg(message))
    }
}

/// The Error type for the registry.
///
/// It can represent any kind of error the registry might encounter.
#[derive(Error, Debug)]
pub enum Error {
    /// An IO error (file not found, access forbidden, etc...).
    #[error("IO error: {0}")]
    IOError(#[from] IOError),
    /// An error while parsing a URL.
    #[error("URL parse error: {0}")]
    UrlParseError(#[from] url::ParseError),
    /// JSON (de)serialization error (invalid JSON parsed, etc...).
    #[error("JSON error: {0}")]
    JSONError(#[from] JSONError),
    /// TOML (de)serialization error (invalid TOML parsed, etc...).
    #[error("TOML error: {0}")]
    TOMLError(#[from] TOMLError),
    /// SQL error (invalid queries, database disconnections, etc...).
    #[error("SQL error: {0}")]
    SQLError(#[from] SQLError),
    /// Version parsing/requirement errors (invalid version format parsed, etc...).
    #[error("semver error: {0}")]
    SemverError(#[from] SemverError),
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
    /// Tantivy's errors.
    #[error("{0}")]
    TantivyError(#[from] TantivyError),
    /// Open directory error
    #[error("{0}")]
    OpenDirectoryError(#[from] OpenDirectoryError),
    /// Empty stop words.
    #[error("Empty stop word filter")]
    EmptyStopWord,
    /// Tantivy's index is poisoned
    #[error("Tantivy's index is poisoned: {0}")]
    PoisonedError(String),
    /// Missing id field or on of nae's field in index schema
    /// Should never happen...
    #[error("Missing {0} in Tantivy's schema")]
    MissingField(&'static str),
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
    /// The uploaded crate is larger than the maximum allowed crate size.
    #[error(
        "uploaded crate is larger than the maximum allowed crate size of {max_crate_size} bytes"
    )]
    CrateTooLarge {
        /// The maximum allowed crate size (in bytes).
        max_crate_size: u64,
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
