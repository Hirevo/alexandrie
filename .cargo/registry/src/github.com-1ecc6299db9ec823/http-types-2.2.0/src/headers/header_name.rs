use std::borrow::Cow;
use std::fmt::{self, Debug, Display};
use std::str::FromStr;

use crate::Error;

/// A header name.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct HeaderName(Cow<'static, str>);

impl HeaderName {
    /// Create a new `HeaderName` from a Vec of ASCII bytes.
    ///
    /// # Error
    ///
    /// This function will error if the bytes is not valid ASCII.
    pub fn from_bytes(mut bytes: Vec<u8>) -> Result<Self, Error> {
        crate::ensure!(bytes.is_ascii(), "Bytes should be valid ASCII");
        bytes.make_ascii_lowercase();

        // This is permitted because ASCII is valid UTF-8, and we just checked that.
        let string = unsafe { String::from_utf8_unchecked(bytes.to_vec()) };
        Ok(HeaderName(Cow::Owned(string)))
    }

    /// Returns the header name as a `&str`.
    pub fn as_str(&self) -> &'_ str {
        &self.0
    }

    /// Converts a vector of bytes to a `HeaderName` without checking that the string contains
    /// valid ASCII.
    ///
    /// # Safety
    ///
    /// This function is unsafe because it does not check that the bytes passed to it are valid
    /// ASCII. If this constraint is violated, it may cause memory
    /// unsafety issues with future users of the HeaderName, as the rest of the library assumes
    /// that Strings are valid ASCII.
    pub unsafe fn from_bytes_unchecked(mut bytes: Vec<u8>) -> Self {
        bytes.make_ascii_lowercase();
        let string = String::from_utf8_unchecked(bytes);
        HeaderName(Cow::Owned(string))
    }

    /// Converts a string assumed to lowercase into a `HeaderName`
    pub(crate) const fn from_lowercase_str(str: &'static str) -> Self {
        HeaderName(Cow::Borrowed(str))
    }
}

impl Debug for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Display for HeaderName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for HeaderName {
    type Err = Error;

    /// Create a new `HeaderName`.
    ///
    /// This checks it's valid ASCII, and lowercases it.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        crate::ensure!(s.is_ascii(), "String slice should be valid ASCII");
        Ok(HeaderName(Cow::Owned(s.to_ascii_lowercase())))
    }
}

impl From<&HeaderName> for HeaderName {
    fn from(value: &HeaderName) -> HeaderName {
        value.clone()
    }
}

impl<'a> From<&'a str> for HeaderName {
    fn from(value: &'a str) -> Self {
        Self::from_str(value).expect("String slice should be valid ASCII")
    }
}

impl PartialEq<str> for HeaderName {
    fn eq(&self, other: &str) -> bool {
        match HeaderName::from_str(other) {
            Err(_) => false,
            Ok(other) => self == &other,
        }
    }
}

impl<'a> PartialEq<&'a str> for HeaderName {
    fn eq(&self, other: &&'a str) -> bool {
        match HeaderName::from_str(other) {
            Err(_) => false,
            Ok(other) => self == &other,
        }
    }
}

impl PartialEq<String> for HeaderName {
    fn eq(&self, other: &String) -> bool {
        match HeaderName::from_str(&other) {
            Err(_) => false,
            Ok(other) => self == &other,
        }
    }
}

impl<'a> PartialEq<&String> for HeaderName {
    fn eq(&self, other: &&String) -> bool {
        match HeaderName::from_str(other) {
            Err(_) => false,
            Ok(other) => self == &other,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_name_static_non_static() {
        let static_header = HeaderName::from_lowercase_str("hello");
        let non_static_header = HeaderName::from_str("hello").unwrap();

        assert_eq!(&static_header, &non_static_header);
        assert_eq!(&static_header, &static_header);
        assert_eq!(&non_static_header, &non_static_header);

        assert_eq!(static_header, non_static_header);
        assert_eq!(static_header, static_header);
        assert_eq!(non_static_header, non_static_header);
    }

    #[test]
    fn equality() {
        let static_header = HeaderName::from_lowercase_str("hello");
        assert_eq!(static_header, "hello");
        assert_eq!(&static_header, "hello");
        assert_eq!(static_header, String::from("hello"));
        assert_eq!(static_header, &String::from("hello"));

        // Must validate regardless of casing.
        assert_eq!(static_header, &String::from("Hello"));
    }

    #[test]
    fn pass_name_by_ref() {
        let mut res = crate::Response::new(200);
        res.insert_header(&crate::headers::HOST, "127.0.0.1");
    }

    #[test]
    fn test_debug() {
        let header_name = HeaderName::from_str("hello").unwrap();
        assert_eq!(format!("{:?}", header_name), "\"hello\"");
    }
}
