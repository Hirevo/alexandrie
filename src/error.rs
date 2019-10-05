use std::convert::TryInto;
use std::error;
use std::fmt;
use std::io;

use diesel::result::Error as SQLError;
use io::Error as IOError;
use json::Error as JSONError;
use semver::{SemVerError as SemverError, Version};
use toml::de::Error as TOMLError;
// use tide::Error as TideError;

use json::json;
use tide::response::IntoResponse;
use tide::Response;

use crate::db::models::Author;

/// The Error type for the registry.
///  
/// It can represent any kind of error the registry might encounter.
#[derive(Debug)]
pub enum Error {
    /// An I/O error (file not found, access forbidden, etc...).
    IOError(IOError),
    /// JSON (de)serialization error (invalid JSON parsed, etc...).
    JSONError(JSONError),
    /// TOML (de)serialization error (invalid TOML parsed, etc...).
    TOMLError(TOMLError),
    /// SQL error (invalid queries, database disconnections, etc...).
    SQLError(SQLError),
    /// Version parsing errors (invalid version format parsed, etc...).
    SemverError(SemverError),
    /// Tide error (invalid query params, could not keep up with the rising tide, etc...).
    // TideError(TideError),
    /// Alexandrie's custom errors (crate not found, invalid token, etc...).
    AlexError(AlexError),
}

/// The Error type for Alexandrie's own errors.
#[derive(Debug)]
pub enum AlexError {
    /// The requested crate cannot be found.
    CrateNotFound(String),
    /// The crate is not owned by the user.
    CrateNotOwned(String, Author),
    /// The published crate version is lower than the current hosted version.
    VersionTooLow {
        /// The krate's name.
        krate: String,
        /// The available hosted version.
        hosted: Version,
        /// The proposed version to be published.
        published: Version,
    },
    /// The token used to access the registry is invalid.
    InvalidToken,
    /// The request is invalid because of a required query parameter.
    MissingQueryParams(&'static [&'static str]),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let message = match self {
            Error::IOError(_) => "internal server error".to_string(),
            Error::JSONError(_) => "internal server error".to_string(),
            Error::TOMLError(_) => "internal server error".to_string(),
            Error::SQLError(_) => "internal server error".to_string(),
            Error::SemverError(_) => "internal server error".to_string(),
            Error::AlexError(err) => err.to_string(),
        };

        let mut response = tide::response::json(json!({
            "errors": [{
                "detail": message,
            }]
        }));
        *response.status_mut() = tide::http::StatusCode::INTERNAL_SERVER_ERROR;
        response
    }
}

impl fmt::Display for AlexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlexError::CrateNotFound(name) => write!(f, "no crate named '{0}' found", name),
            AlexError::CrateNotOwned(name, _) => write!(f, "you are not an owner of '{0}'", name),
            AlexError::VersionTooLow {
                hosted, published, ..
            } => write!(
                f,
                "the published version is too low (hosted version is {1}, {0} <= {1})",
                published, hosted,
            ),
            AlexError::InvalidToken => write!(f, "invalid token"),
            AlexError::MissingQueryParams(params) => {
                write!(f, "missing query parameters: {0:?}", params)
            }
        }
    }
}

impl error::Error for AlexError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(err) => err.fmt(f),
            Error::JSONError(err) => err.fmt(f),
            Error::TOMLError(err) => err.fmt(f),
            Error::SQLError(err) => err.fmt(f),
            Error::SemverError(err) => err.fmt(f),
            Error::AlexError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IOError(err) => err.source(),
            Error::JSONError(err) => err.source(),
            Error::TOMLError(err) => err.source(),
            Error::SQLError(err) => err.source(),
            Error::SemverError(err) => err.source(),
            Error::AlexError(err) => err.source(),
        }
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Error {
        Error::IOError(err)
    }
}

impl From<JSONError> for Error {
    fn from(err: JSONError) -> Error {
        Error::JSONError(err)
    }
}

impl From<toml::de::Error> for Error {
    fn from(err: toml::de::Error) -> Error {
        Error::TOMLError(err)
    }
}

impl From<SQLError> for Error {
    fn from(err: SQLError) -> Error {
        Error::SQLError(err)
    }
}

impl From<SemverError> for Error {
    fn from(err: SemverError) -> Error {
        Error::SemverError(err)
    }
}

impl From<AlexError> for Error {
    fn from(err: AlexError) -> Error {
        Error::AlexError(err)
    }
}

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
