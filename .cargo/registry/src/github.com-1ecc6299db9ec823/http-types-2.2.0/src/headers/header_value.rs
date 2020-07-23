use std::convert::TryFrom;
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

use crate::headers::HeaderValues;
use crate::Error;
use crate::{Cookie, Mime};

/// A header value.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct HeaderValue {
    inner: String,
}

impl HeaderValue {
    /// Create a new `HeaderValue` from a Vec of ASCII bytes.
    ///
    /// # Error
    ///
    /// This function will error if the bytes is not valid ASCII.
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, Error> {
        crate::ensure!(bytes.is_ascii(), "Bytes should be valid ASCII");

        // This is permitted because ASCII is valid UTF-8, and we just checked that.
        let string = unsafe { String::from_utf8_unchecked(bytes) };
        Ok(Self { inner: string })
    }

    /// Converts a vector of bytes to a `HeaderValue` without checking that the string contains
    /// valid ASCII.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed to it are valid
    /// ASCII. If this constraint is violated, it may cause memory
    /// unsafety issues with future users of the HeaderValue, as the rest of the library assumes
    /// that Strings are valid ASCII.
    pub unsafe fn from_bytes_unchecked(bytes: Vec<u8>) -> Self {
        let string = String::from_utf8_unchecked(bytes);
        Self { inner: string }
    }

    /// Get the header value as a `&str`
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<Mime> for HeaderValue {
    fn from(mime: Mime) -> Self {
        HeaderValue {
            inner: format!("{}", mime),
        }
    }
}

impl From<Cookie<'_>> for HeaderValue {
    fn from(cookie: Cookie<'_>) -> Self {
        HeaderValue {
            inner: cookie.to_string(),
        }
    }
}

impl From<&Mime> for HeaderValue {
    fn from(mime: &Mime) -> Self {
        HeaderValue {
            inner: format!("{}", mime),
        }
    }
}

impl FromStr for HeaderValue {
    type Err = Error;

    /// Create a new `HeaderValue`.
    ///
    /// This checks it's valid ASCII.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::ensure!(s.is_ascii(), "String slice should be valid ASCII");
        Ok(Self {
            inner: String::from(s),
        })
    }
}

impl<'a> TryFrom<&'a str> for HeaderValue {
    type Error = Error;

    fn try_from(value: &'a str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl Debug for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.inner)
    }
}

impl Display for HeaderValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl PartialEq<str> for HeaderValue {
    fn eq(&self, other: &str) -> bool {
        self.inner == other
    }
}

impl<'a> PartialEq<&'a str> for HeaderValue {
    fn eq(&self, other: &&'a str) -> bool {
        &self.inner == other
    }
}

impl PartialEq<String> for HeaderValue {
    fn eq(&self, other: &String) -> bool {
        &self.inner == other
    }
}

impl<'a> PartialEq<&String> for HeaderValue {
    fn eq(&self, other: &&String) -> bool {
        &&self.inner == other
    }
}

impl From<HeaderValues> for HeaderValue {
    fn from(mut other: HeaderValues) -> Self {
        other.inner.reverse();
        other
            .inner
            .pop()
            .expect("HeaderValues should contain at least one value")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug() {
        let header_value = HeaderValue::from_str("foo0").unwrap();
        assert_eq!(format!("{:?}", header_value), "\"foo0\"");
    }
}
