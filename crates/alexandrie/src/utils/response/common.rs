use axum::http::StatusCode;
use axum_extra::response::Html;

use crate::config::AppState;
use crate::db::models::Author;
use crate::error::FrontendError;

/// Constructs a response for 'unauthenticated-only' pages.
pub fn already_logged_in(
    state: &AppState,
    user: Author,
) -> Result<(StatusCode, Html<String>), FrontendError> {
    let rendered = super::error_html(state, Some(user), "You are already logged in.")?;
    Ok((StatusCode::UNAUTHORIZED, Html(rendered)))
}

/// Constructs a response for 'authenticated-only' pages.
pub fn need_to_login(state: &AppState) -> Result<(StatusCode, Html<String>), FrontendError> {
    let rendered = super::error_html(state, None, "You need to login first.")?;
    Ok((StatusCode::UNAUTHORIZED, Html(rendered)))
}
