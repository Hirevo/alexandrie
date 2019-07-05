use std::convert::TryInto;
use std::error;
use std::fmt;
use std::io;

use semver::{Version, SemVerError as SemverError};
use json::Error as JSONError;
use io::Error as IOError;
use git2::Error as GitError;
use rocket::config::ConfigError as ConfigError;
use diesel::result::Error as SQLError;
use toml::de::Error as TOMLError;

/// The Error type for the registry.  
/// It can represent any kind of error the registry might encounter.
#[derive(Debug)]
pub enum Error {
    /// An I/O error (file not found, access forbidden, etc...).
    IOError(IOError),
    /// Git error (currently unused).
    GitError(GitError),
    /// JSON (de)serialization error (invalid JSON parsed, etc...).
    JSONError(JSONError),
    /// TOML (de)serialization error (invalid TOML parsed, etc...).
    TOMLError(TOMLError),
    /// SQL error (invalid queries, database disconnections, etc...).
    SQLError(SQLError),
    /// Version parsing errors (invalid version format parsed, etC...).
    SemverError(SemverError),
    /// A configuration error (invalid Rocket.toml file, etc...).
    ConfigError(ConfigError),
    /// Alexandrie's custom errors (crate not found, invalid token, etc...).
    AlexError(AlexError),
}

/// The Error type for Alexandrie's own errors.
#[derive(Debug)]
pub enum AlexError {
    /// The requested crate cannot be found.
    CrateNotFound(String),
    /// The published crate version is lower than the current hosted version.
    VersionTooLow { krate: String, hosted: Version, published: Version },
    /// The token used to access the registry is invalid.
    InvalidToken,
}

impl fmt::Display for AlexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlexError::CrateNotFound(name) => write!(f, "no crate named '{}' found", name),
            AlexError::VersionTooLow{ .. } => write!(
                f,
                "the published version is not greater than the existing one"
            ),
            AlexError::InvalidToken => write!(f, "invalid token"),
        }
    }
}

impl error::Error for AlexError {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::IOError(err) => err.fmt(f),
            Error::GitError(err) => err.fmt(f),
            Error::JSONError(err) => err.fmt(f),
            Error::TOMLError(err) => err.fmt(f),
            Error::SQLError(err) => err.fmt(f),
            Error::SemverError(err) => err.fmt(f),
            Error::ConfigError(err) => err.fmt(f),
            Error::AlexError(err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::IOError(err) => err.source(),
            Error::GitError(err) => err.source(),
            Error::JSONError(err) => err.source(),
            Error::TOMLError(err) => err.source(),
            Error::SQLError(err) => err.source(),
            Error::SemverError(err) => err.source(),
            Error::ConfigError(err) => err.source(),
            Error::AlexError(err) => err.source(),
        }
    }
}

impl From<IOError> for Error {
    fn from(err: IOError) -> Error {
        Error::IOError(err)
    }
}

impl From<GitError> for Error {
    fn from(err: GitError) -> Error {
        Error::GitError(err)
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

impl From<ConfigError> for Error {
    fn from(err: ConfigError) -> Error {
        Error::ConfigError(err)
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

impl TryInto<GitError> for Error {
    type Error = ();

    fn try_into(self) -> Result<GitError, Self::Error> {
        match self {
            Error::GitError(err) => Ok(err),
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

impl TryInto<ConfigError> for Error {
    type Error = ();

    fn try_into(self) -> Result<ConfigError, Self::Error> {
        match self {
            Error::ConfigError(err) => Ok(err),
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
