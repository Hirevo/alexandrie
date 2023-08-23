use std::fmt;

use axum::headers::authorization::Credentials;
use axum::http::header::HeaderValue;
use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};

use crate::error::ApiError;

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "frontend")]
pub use self::frontend::*;

/// Authorization header's name.
pub const AUTHORIZATION_HEADER: &str = "authorization";

/// A value that is both a valid `HeaderValue` and `String`.
///
/// This struct mimics the internal `HeaderValueString` type that the `headers` crate uses.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct HeaderValueString(
    /// Care must be taken to only set this value when it is also
    /// a valid `String`, since `as_str` will convert to a `&str`
    /// in an unchecked manner.
    HeaderValue,
);

impl HeaderValueString {
    pub(crate) fn from_value(val: &HeaderValue) -> Result<Self, ApiError> {
        if val.to_str().is_ok() {
            Ok(HeaderValueString(val.clone()))
        } else {
            Err(ApiError::msg("invalid `Authorization` header"))
        }
    }

    pub(crate) fn as_str(&self) -> &str {
        // HeaderValueString is only created from HeaderValues
        // that have validated they are also UTF-8 strings.
        unsafe { std::str::from_utf8_unchecked(self.0.as_bytes()) }
    }
}

impl fmt::Debug for HeaderValueString {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}

#[derive(Clone, PartialEq, Debug)]
/// Represent a bare token from the `Authorization` header's value.
pub struct Bare(HeaderValueString);

impl Bare {
    /// View the token part as a `&str`.
    pub fn token(&self) -> &str {
        self.0.as_str()
    }
}

impl Credentials for Bare {
    const SCHEME: &'static str = "";

    fn decode(value: &HeaderValue) -> Option<Self> {
        HeaderValueString::from_value(value).ok().map(Self)
    }

    fn encode(&self) -> HeaderValue {
        (&self.0).0.clone()
    }
}

/// Generates a new random registry token (as a hex-encoded SHA-512 digest).
pub fn generate_token() -> String {
    let mut data = [0u8; 16];
    let rng = SystemRandom::new();
    rng.fill(&mut data).unwrap();
    hex::encode(hasher::digest(&hasher::SHA512, data.as_ref()))
}
