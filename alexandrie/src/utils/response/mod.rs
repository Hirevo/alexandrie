use json::json;
use serde::Serialize;
use tide::http::mime;
use tide::{Body, Response, StatusCode};

/// Various utilities to construct common response pages.
#[cfg(feature = "frontend")]
pub mod common;

#[cfg(feature = "frontend")]
use crate::config::State;
#[cfg(feature = "frontend")]
use crate::db::models::Author;

/// Constructs a HTML response with the provided body.
pub fn html(body: String) -> Response {
    self::html_with_status(StatusCode::Ok, body)
}

/// Constructs a HTML response with the provided body and status code.
pub fn html_with_status(status: StatusCode, body: String) -> Response {
    let mut response = Response::new(status);
    response.set_body(Body::from_string(body));
    response.set_content_type(mime::HTML);
    response
}

/// Constructs a JSON response with the provided body.
pub fn json(body: &impl Serialize) -> Response {
    self::json_with_status(StatusCode::Ok, body)
}

/// Constructs a JSON response with the provided body and status code.
pub fn json_with_status(status: StatusCode, body: &impl Serialize) -> Response {
    let mut response = Response::new(status);
    response.set_body(Body::from_json(body).unwrap());
    response
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
) -> tide::Result {
    let engine = &state.frontend.handlebars;
    let context = json!({
        "user": user,
        "instance": &state.frontend.config,
        "error_msg": error_msg.as_ref(),
    });
    Ok(self::html_with_status(
        status,
        engine.render("error", &context)?,
    ))
}

/// Constructs a redirection response (302 Found) to the specified URL.
pub fn redirect(url: &str) -> Response {
    let mut response = Response::new(StatusCode::Found);
    response.insert_header("location", url);
    response
}
