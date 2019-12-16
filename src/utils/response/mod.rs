use http::status::StatusCode;
use json::json;
use serde::Serialize;
use tide::Response;

/// Various utilities to construct common response pages.
#[cfg(feature = "frontend")]
pub mod common;

#[cfg(feature = "frontend")]
use crate::config::State;
#[cfg(feature = "frontend")]
use crate::db::models::Author;

/// Constructs a HTML response with the provided body.
pub fn html(body: String) -> Response {
    self::html_with_status(StatusCode::OK, body)
}

/// Constructs a HTML response with the provided body and status code.
pub fn html_with_status(status: StatusCode, body: String) -> Response {
    Response::new(status.as_u16())
        .body_string(body)
        .set_header("content-type", "text/html")
}

/// Constructs a JSON response with the provided body.
pub fn json(body: &impl Serialize) -> Response {
    self::json_with_status(StatusCode::OK, body)
}

/// Constructs a JSON response with the provided body and status code.
pub fn json_with_status(status: StatusCode, body: &impl Serialize) -> Response {
    Response::new(status.as_u16()).body_json(body).unwrap()
}

/// Constructs an API error (JSON) response with the specified status code and error message.
pub fn error(status: StatusCode, error_msg: impl AsRef<str>) -> Response {
    let error_msg = error_msg.as_ref();
    let data = json!({
        "errors": [{ "details": error_msg }]
    });
    self::json_with_status(status, &data)
}

/// Construct an HTML error response (used for the frontend).
#[cfg(feature = "frontend")]
pub fn error_html(
    state: &State,
    user: Option<Author>,
    status: StatusCode,
    error_msg: impl AsRef<str>,
) -> Response {
    let engine = &state.frontend.handlebars;
    let context = json!({
        "user": user,
        "instance": &state.frontend.config,
        "error_msg": error_msg.as_ref(),
    });
    self::html_with_status(status, engine.render("error", &context).unwrap())
}

/// Constructs a redirection response (302 Found) to the specified URL.
pub fn redirect(url: &str) -> Response {
    Response::new(StatusCode::FOUND.as_u16()).set_header("location", url)
}
