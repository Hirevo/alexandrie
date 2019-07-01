use std::io;
use std::fmt;
use std::error;
use std::convert::TryInto;

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    GitError(git2::Error),
    JSONError(json::Error),
    AlexError(AlexError),
}

#[derive(Debug)]
pub enum AlexError {
    CrateNotFound(String),
}

impl fmt::Display for AlexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AlexError::CrateNotFound(name) => write!(f, "error: no crate named '{}' found", name),
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
            Error::AlexError(err) => err.source(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Error {
        Error::IOError(err)
    }
}

impl From<git2::Error> for Error {
    fn from(err: git2::Error) -> Error {
        Error::GitError(err)
    }
}

impl From<json::Error> for Error {
    fn from(err: json::Error) -> Error {
        Error::JSONError(err)
    }
}

impl From<AlexError> for Error {
    fn from(err: AlexError) -> Error {
        Error::AlexError(err)
    }
}

impl TryInto<io::Error> for Error {
    type Error = ();

    fn try_into(self) -> Result<io::Error, Self::Error> {
        match self {
            Error::IOError(err) => Ok(err),
            _ => Err(()),
        }
    }
}

impl TryInto<git2::Error> for Error {
    type Error = ();

    fn try_into(self) -> Result<git2::Error, Self::Error> {
        match self {
            Error::GitError(err) => Ok(err),
            _ => Err(()),
        }
    }
}

impl TryInto<json::Error> for Error {
    type Error = ();

    fn try_into(self) -> Result<json::Error, Self::Error> {
        match self {
            Error::JSONError(err) => Ok(err),
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
