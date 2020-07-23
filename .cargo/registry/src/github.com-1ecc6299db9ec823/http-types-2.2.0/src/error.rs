//! HTTP error types

use std::error::Error as StdError;
use std::fmt::{self, Debug, Display};

use crate::StatusCode;

/// A specialized `Result` type for HTTP operations.
///
/// This type is broadly used across `http_types` for any operation which may
/// produce an error.
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for HTTP operations.
pub struct Error {
    error: anyhow::Error,
    status: crate::StatusCode,
}

impl Error {
    /// Create a new error object from any error type.
    ///
    /// The error type must be threadsafe and 'static, so that the Error will be
    /// as well. If the error type does not provide a backtrace, a backtrace will
    /// be created here to ensure that a backtrace exists.
    pub fn new<E>(status: StatusCode, error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self {
            status,
            error: anyhow::Error::new(error),
        }
    }

    /// Create a new error object from static string.
    pub fn from_str<M>(status: StatusCode, msg: M) -> Self
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Self {
            status,
            error: anyhow::Error::msg(msg),
        }
    }

    /// Create a new error from a message.
    pub(crate) fn new_adhoc<M>(message: M) -> Error
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Self::from_str(StatusCode::InternalServerError, message)
    }

    /// Get the status code associated with this error.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// Set the status code associated with this error.
    pub fn set_status(&mut self, status: StatusCode) {
        self.status = status;
    }

    /// Get the backtrace for this Error.
    ///
    /// Backtraces are only available on the nightly channel. Tracking issue:
    /// [rust-lang/rust#53487][tracking].
    ///
    /// In order for the backtrace to be meaningful, the environment variable
    /// `RUST_LIB_BACKTRACE=1` must be defined. Backtraces are somewhat
    /// expensive to capture in Rust, so we don't necessarily want to be
    /// capturing them all over the place all the time.
    ///
    /// [tracking]: https://github.com/rust-lang/rust/issues/53487
    #[cfg(backtrace)]
    pub fn backtrace(&self) -> &std::backtrace::Backtrace {
        self.error.downcast_ref::<E>()
    }

    /// Attempt to downcast the error object to a concrete type.
    pub fn downcast<E>(self) -> std::result::Result<E, Self>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        if self.error.downcast_ref::<E>().is_some() {
            Ok(self.error.downcast().unwrap())
        } else {
            Err(self)
        }
    }

    /// Downcast this error object by reference.
    pub fn downcast_ref<E>(&self) -> Option<&E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.error.downcast_ref::<E>()
    }

    /// Downcast this error object by mutable reference.
    pub fn downcast_mut<E>(&mut self) -> Option<&mut E>
    where
        E: Display + Debug + Send + Sync + 'static,
    {
        self.error.downcast_mut::<E>()
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.error)
    }
}

impl Debug for Error {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.error)
    }
}

impl<E> From<E> for Error
where
    E: StdError + Send + Sync + 'static,
{
    fn from(error: E) -> Self {
        Self {
            error: anyhow::Error::new(error),
            status: StatusCode::InternalServerError,
        }
    }
}

impl AsRef<dyn StdError + Send + Sync> for Error {
    fn as_ref(&self) -> &(dyn StdError + Send + Sync + 'static) {
        self.error.as_ref()
    }
}

impl AsRef<StatusCode> for Error {
    fn as_ref(&self) -> &StatusCode {
        &self.status
    }
}

impl AsMut<StatusCode> for Error {
    fn as_mut(&mut self) -> &mut StatusCode {
        &mut self.status
    }
}

impl AsRef<dyn StdError> for Error {
    fn as_ref(&self) -> &(dyn StdError + 'static) {
        self.error.as_ref()
    }
}

impl From<Error> for Box<dyn StdError + Send + Sync + 'static> {
    fn from(error: Error) -> Self {
        error.error.into()
    }
}

impl From<Error> for Box<dyn StdError + 'static> {
    fn from(error: Error) -> Self {
        Box::<dyn StdError + Send + Sync>::from(error.error)
    }
}
