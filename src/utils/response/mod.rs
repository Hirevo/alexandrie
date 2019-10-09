use bytes::Bytes;
use http::status::StatusCode;
use json::json;
use tide::{Body, Response};

/// Various utilities to construct common response pages.
#[cfg(feature = "frontend")]
pub mod common;

#[cfg(feature = "frontend")]
use crate::config::State;
#[cfg(feature = "frontend")]
use crate::db::models::Author;

/// Constructs a HTML response with the provided body.
pub fn html(body: impl Send + Into<Bytes>) -> Response {
    http::Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html")
        .body(Body::from(body))
        .unwrap()
}

/// Constructs a HTML response with the provided body and status code.
pub fn html_with_status(status: StatusCode, body: impl Send + Into<Bytes>) -> Response {
    http::Response::builder()
        .status(status)
        .header("content-type", "text/html")
        .body(Body::from(body))
        .unwrap()
}

/// Constructs an API error (JSON) response with the specified status code and error message.
pub fn error(status: StatusCode, error_msg: impl AsRef<str>) -> Response {
    let error_msg = error_msg.as_ref();
    let mut response = tide::response::json(json!({
        "errors": [{ "details": error_msg }]
    }));
    *response.status_mut() = status;
    response
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
    http::Response::builder()
        .status(StatusCode::FOUND)
        .header("location", url)
        .body(Body::empty())
        .unwrap()
}
