use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::headers::Header;
use axum::http::header::{HeaderName, HeaderValue, AUTHORIZATION};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{async_trait, RequestPartsExt, TypedHeader};

use crate::config::AppState;
use crate::db::models::Author;
use crate::utils;

/// The authentication extractor for the programmatic API of `alexandrie`.
///
/// What it does:
///   - extracts the author token from the `Authorization` header.
///   - tries to match it with an existing author in the database.
///   - exposes the [`Author`] struct if successful.
pub struct Auth(pub Author);

impl Auth {
    /// Unwraps the inner `Author` struct
    pub fn into_inner(self) -> Author {
        self.0
    }
}

impl Deref for Auth {
    type Target = Author;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Auth {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[async_trait]
impl FromRequestParts<Arc<AppState>> for Auth {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(authorization) = parts
            .extract::<TypedHeader<Authorization>>()
            .await
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        let token = authorization.token().to_string();

        let author = state
            .db
            .run(move |conn| {
                //? Get the author matching the ID from the session.
                utils::checks::get_author(conn, token)
            })
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::UNAUTHORIZED)?;

        Ok(Auth(author))
    }
}

/// A value that is both a valid `HeaderValue` and `String`.
///
/// This struct mimics the internal `HeaderValueString` type that the `headers` crate uses.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HeaderValueString(
    /// Care must be taken to only set this value when it is also
    /// a valid `String`, since `as_str` will convert to a `&str`
    /// in an unchecked manner.
    HeaderValue,
);

impl HeaderValueString {
    pub(crate) fn from_value(val: &HeaderValue) -> Result<Self, axum::headers::Error> {
        if val.to_str().is_ok() {
            Ok(HeaderValueString(val.clone()))
        } else {
            Err(axum::headers::Error::invalid())
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
struct Authorization(HeaderValueString);

impl Authorization {
    /// View the token part as a `&str`.
    pub fn token(&self) -> &str {
        self.0.as_str()
    }
}

impl Header for Authorization {
    fn name() -> &'static HeaderName {
        &AUTHORIZATION
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, axum::headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        values
            .next()
            .ok_or_else(axum::headers::Error::invalid)
            .and_then(HeaderValueString::from_value)
            .map(Self)
    }

    fn encode<E>(&self, values: &mut E)
    where
        E: Extend<HeaderValue>,
    {
        let value = (&self.0).0.clone();
        values.extend(std::iter::once(value));
    }
}
