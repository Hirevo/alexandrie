use crate::{Error, StatusCode};
use core::convert::{Infallible, TryInto};
use std::error::Error as StdError;
use std::fmt::Debug;

/// Provides the `status` method for `Result` and `Option`.
///
/// This trait is sealed and cannot be implemented outside of `http-types`.
pub trait Status<T, E>: private::Sealed {
    /// Wrap the error value with an additional status code.
    fn status<S>(self, status: S) -> Result<T, Error>
    where
        S: TryInto<StatusCode>,
        S::Error: Debug;

    /// Wrap the error value with an additional status code that is evaluated
    /// lazily only once an error does occur.
    fn with_status<S, F>(self, f: F) -> Result<T, Error>
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
        F: FnOnce() -> S;
}

impl<T, E> Status<T, E> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    /// Wrap the error value with an additional status code.
    ///
    /// # Panics
    ///
    /// Panics if [`Status`][status] is not a valid [`StatusCode`][statuscode].
    ///
    /// [status]: crate::Status
    /// [statuscode]: crate::StatusCode
    fn status<S>(self, status: S) -> Result<T, Error>
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        self.map_err(|error| {
            let status = status
                .try_into()
                .expect("Could not convert into a valid `StatusCode`");
            Error::new(status, error)
        })
    }

    /// Wrap the error value with an additional status code that is evaluated
    /// lazily only once an error does occur.
    ///
    /// # Panics
    ///
    /// Panics if [`Status`][status] is not a valid [`StatusCode`][statuscode].
    ///
    /// [status]: crate::Status
    /// [statuscode]: crate::StatusCode
    fn with_status<S, F>(self, f: F) -> Result<T, Error>
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
        F: FnOnce() -> S,
    {
        self.map_err(|error| {
            let status = f()
                .try_into()
                .expect("Could not convert into a valid `StatusCode`");
            Error::new(status, error)
        })
    }
}

impl<T> Status<T, Infallible> for Option<T> {
    /// Wrap the error value with an additional status code.
    ///
    /// # Panics
    ///
    /// Panics if [`Status`][status] is not a valid [`StatusCode`][statuscode].
    ///
    /// [status]: crate::Status
    /// [statuscode]: crate::StatusCode
    fn status<S>(self, status: S) -> Result<T, Error>
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
    {
        self.ok_or_else(|| {
            let status = status
                .try_into()
                .expect("Could not convert into a valid `StatusCode`");
            Error::from_str(status, "NoneError")
        })
    }

    /// Wrap the error value with an additional status code that is evaluated
    /// lazily only once an error does occur.
    ///
    /// # Panics
    ///
    /// Panics if [`Status`][status] is not a valid [`StatusCode`][statuscode].
    ///
    /// [status]: crate::Status
    /// [statuscode]: crate::StatusCode
    fn with_status<S, F>(self, f: F) -> Result<T, Error>
    where
        S: TryInto<StatusCode>,
        S::Error: Debug,
        F: FnOnce() -> S,
    {
        self.ok_or_else(|| {
            let status = f()
                .try_into()
                .expect("Could not convert into a valid `StatusCode`");
            Error::from_str(status, "NoneError")
        })
    }
}

pub(crate) mod private {
    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> {}
    impl<T> Sealed for Option<T> {}
}

#[cfg(test)]
mod test {
    use super::Status;

    #[test]
    fn construct_shorthand_with_valid_status_code() {
        let _res = Some(()).status(200).unwrap();
    }

    #[test]
    #[should_panic(expected = "Could not convert into a valid `StatusCode`")]
    fn construct_shorthand_with_invalid_status_code() {
        let res: Result<(), std::io::Error> =
            Err(std::io::Error::new(std::io::ErrorKind::Other, "oh no!"));
        let _res = res.status(600).unwrap();
    }
}
