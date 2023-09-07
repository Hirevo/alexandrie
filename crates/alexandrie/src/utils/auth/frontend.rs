use std::ops::{Deref, DerefMut};
use std::sync::Arc;

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::{async_trait, RequestPartsExt};
use axum_sessions::extractors::ReadableSession;
use diesel::prelude::*;

use crate::config::AppState;
use crate::db::models::Author;
use crate::db::schema::*;

/// Session cookie's name.
pub const COOKIE_NAME: &str = "session";

/// The authentication extractor for the frontend of `alexandrie`.
///
/// What it does:
///   - extracts the author ID from the session data.
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
        let session = parts
            .extract::<ReadableSession>()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

        let author_id: i64 = session.get("author.id").ok_or(StatusCode::BAD_REQUEST)?;

        let author = state
            .db
            .run(move |conn| {
                //? Get the author matching the ID from the session.
                authors::table.find(author_id).first(conn).optional()
            })
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)?;

        Ok(Auth(author))
    }
}
