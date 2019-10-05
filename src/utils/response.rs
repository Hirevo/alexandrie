use bytes::Bytes;
use json::json;
use tide::http::status::StatusCode;
use tide::{Body, Response};

pub(crate) fn html(body: impl Send + Into<Bytes>) -> Response {
    tide::http::Response::builder()
        .status(tide::http::StatusCode::OK)
        .header("content-type", "text/html")
        .body(Body::from(body))
        .unwrap()
}

pub(crate) fn error(status: StatusCode, error_msg: impl AsRef<str>) -> Response {
    let error_msg = error_msg.as_ref();
    let mut response = tide::response::json(json!({
        "errors": [{ "details": error_msg }]
    }));
    *response.status_mut() = status;
    response
}
