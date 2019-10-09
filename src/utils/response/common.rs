use http::status::StatusCode;
use tide::Response;

use crate::config::State;
use crate::db::models::Author;

/// Constructs a response for 'unauthenticated-only' pages.
pub fn already_logged_in(state: &State, user: Author) -> Response {
    super::error_html(
        state,
        Some(user),
        StatusCode::FORBIDDEN,
        "You are already logged in.",
    )
}

/// Constructs a response for 'authenticated-only' pages.
pub fn need_to_login(state: &State) -> Response {
    super::error_html(
        state,
        None,
        StatusCode::FORBIDDEN,
        "You need to login first.",
    )
}
