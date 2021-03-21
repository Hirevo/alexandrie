use tide::StatusCode;

use crate::config::State;
use crate::db::models::Author;

/// Constructs a response for 'unauthenticated-only' pages.
pub fn already_logged_in(state: &State, user: Author) -> tide::Result {
    super::error_html(
        state,
        Some(user),
        StatusCode::Forbidden,
        "You are already logged in.",
    )
}

/// Constructs a response for 'authenticated-only' pages.
pub fn need_to_login(state: &State) -> tide::Result {
    super::error_html(
        state,
        None,
        StatusCode::Forbidden,
        "You need to login first.",
    )
}
