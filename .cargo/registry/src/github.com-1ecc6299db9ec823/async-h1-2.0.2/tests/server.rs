use async_std::io::Cursor;
use async_std::prelude::*;
use common::TestCase;
use http_types::{mime, Body, Response, StatusCode};

mod common;

#[async_std::test]
async fn test_basic_request() {
    let case = TestCase::new_server(
        "fixtures/request-add-date.txt",
        "fixtures/response-add-date.txt",
    )
    .await;

    async_h1::accept(case.clone(), |_req| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body("");
        Ok(res)
    })
    .await
    .unwrap();

    case.assert().await;
}

#[async_std::test]
async fn test_host() {
    let case = TestCase::new_server(
        "fixtures/request-with-host.txt",
        "fixtures/response-with-host.txt",
    )
    .await;

    async_h1::accept(case.clone(), |req| async move {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(req.url().as_str());
        Ok(res)
    })
    .await
    .unwrap();

    case.assert().await;
}

#[async_std::test]
async fn test_chunked_basic() {
    let case = TestCase::new_server(
        "fixtures/request-chunked-basic.txt",
        "fixtures/response-chunked-basic.txt",
    )
    .await;

    async_h1::accept(case.clone(), |_req| async {
        let mut res = Response::new(StatusCode::Ok);
        res.set_body(Body::from_reader(
            Cursor::new(b"Mozilla")
                .chain(Cursor::new(b"Developer"))
                .chain(Cursor::new(b"Network")),
            None,
        ));
        res.set_content_type(mime::PLAIN);
        Ok(res)
    })
    .await
    .unwrap();

    case.assert().await;
}

#[async_std::test]
async fn test_chunked_echo() {
    let case = TestCase::new_server(
        "fixtures/request-chunked-echo.txt",
        "fixtures/response-chunked-echo.txt",
    )
    .await;

    async_h1::accept(case.clone(), |req| async {
        let ct = req.content_type();
        let body: Body = req.into();

        let mut res = Response::new(StatusCode::Ok);
        res.set_body(body);
        if let Some(ct) = ct {
            res.set_content_type(ct);
        }

        Ok(res)
    })
    .await
    .unwrap();

    case.assert().await;
}

#[async_std::test]
async fn test_unexpected_eof() {
    // We can't predict unexpected EOF, so the response content-length is still 11
    let case = TestCase::new_server(
        "fixtures/request-unexpected-eof.txt",
        "fixtures/response-unexpected-eof.txt",
    )
    .await;

    async_h1::accept(case.clone(), |req| async {
        let mut res = Response::new(StatusCode::Ok);
        let ct = req.content_type();
        let body: Body = req.into();
        res.set_body(body);
        if let Some(ct) = ct {
            res.set_content_type(ct);
        }

        Ok(res)
    })
    .await
    .unwrap();

    case.assert().await;
}

#[async_std::test]
async fn test_invalid_trailer() {
    let case = TestCase::new_server(
        "fixtures/request-invalid-trailer.txt",
        "fixtures/response-invalid-trailer.txt",
    )
    .await;

    async_h1::accept(case.clone(), |req| async {
        let mut res = Response::new(StatusCode::Ok);
        let ct = req.content_type();
        let body: Body = req.into();
        res.set_body(body);
        if let Some(ct) = ct {
            res.set_content_type(ct);
        }

        Ok(res)
    })
    .await
    .unwrap_err();

    assert!(case.read_result().await.is_empty());
}
