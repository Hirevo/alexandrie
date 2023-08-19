use json::json;

use crate::error::FrontendError;

/// Various utilities to construct common response pages.
#[cfg(feature = "frontend")]
pub mod common;

#[cfg(feature = "frontend")]
use crate::config::AppState;
#[cfg(feature = "frontend")]
use crate::db::models::Author;

/// Construct an HTML error response (used for the frontend).
#[cfg(feature = "frontend")]
pub fn error_html(
    state: &AppState,
    user: Option<Author>,
    error_msg: impl AsRef<str>,
) -> Result<String, FrontendError> {
    let engine = &state.frontend.handlebars;
    let context = json!({
        "user": user,
        "instance": &state.frontend.config,
        "error_msg": error_msg.as_ref(),
    });
    let rendered = engine.render("error", &context)?;
    Ok(rendered)
}
