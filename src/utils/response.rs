use bytes::Bytes;
use tide::{Body, Response};

pub(crate) fn html(body: impl Send + Into<Bytes>) -> Response {
    tide::http::Response::builder()
        .status(tide::http::StatusCode::OK)
        .header("content-type", "text/html")
        .body(Body::from(body))
        .unwrap()
}
