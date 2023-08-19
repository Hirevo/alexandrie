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

/// The authentication extractor impl for `alexandrie`.
///
/// What it does:
///   - extracts the author ID from the session data.
///   - tries to match it with an existing author in the database.
///   - exposes the [`Author`] struct if successful.
#[async_trait]
impl FromRequestParts<Arc<AppState>> for Author {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // The `Rejection` associated type for `ReadableSession` is `Infallible`, so this unwrapping is fine.
        let session = parts.extract::<ReadableSession>().await.unwrap();
        let author_id: i64 = session.get("author.id").ok_or(StatusCode::BAD_REQUEST)?;

        state
            .db
            .run(move |conn| {
                //? Get the author matching the ID from the session.
                authors::table.find(author_id).first(conn).optional()
            })
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
            .ok_or(StatusCode::BAD_REQUEST)
    }
}
