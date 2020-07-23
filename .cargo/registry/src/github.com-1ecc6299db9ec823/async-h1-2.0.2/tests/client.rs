use crate::common::fixture_path;
use async_h1::client;
use async_std::fs::File;
use http_types::{headers, Method, Request, StatusCode, Url};

mod common;

use common::TestCase;

#[async_std::test]
async fn test_encode_request_add_date() {
    let case = TestCase::new_client(
        "fixtures/request-add-date.txt",
        "fixtures/response-add-date.txt",
    )
    .await;

    let url = Url::parse("http://localhost:8080").unwrap();
    let mut req = Request::new(Method::Post, url);
    req.set_body("hello");

    let res = client::connect(case.clone(), req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);

    case.assert().await;
}

#[async_std::test]
async fn test_response_no_date() {
    let response_fixture = File::open(fixture_path("fixtures/response-no-date.txt"))
        .await
        .unwrap();

    let res = client::decode(response_fixture).await.unwrap();

    pretty_assertions::assert_eq!(res.header(&headers::DATE).is_some(), true);
}

#[async_std::test]
async fn test_multiple_header_values_for_same_header_name() {
    let response_fixture = File::open(fixture_path("fixtures/response-multiple-cookies.txt"))
        .await
        .unwrap();

    let res = client::decode(response_fixture).await.unwrap();

    pretty_assertions::assert_eq!(res.header(&headers::SET_COOKIE).unwrap().iter().count(), 2);
}

#[async_std::test]
async fn test_response_newlines() {
    let response_fixture = File::open(fixture_path("fixtures/response-newlines.txt"))
        .await
        .unwrap();

    let res = client::decode(response_fixture).await.unwrap();

    pretty_assertions::assert_eq!(
        res[headers::CONTENT_LENGTH]
            .as_str()
            .parse::<usize>()
            .unwrap(),
        78
    );
}

#[async_std::test]
async fn test_encode_request_with_connect() {
    let case = TestCase::new_client(
        "fixtures/request-with-connect.txt",
        "fixtures/response-with-connect.txt",
    )
    .await;

    let url = Url::parse("https://example.com:443").unwrap();
    let req = Request::new(Method::Connect, url);

    let res = client::connect(case.clone(), req).await.unwrap();
    assert_eq!(res.status(), StatusCode::Ok);

    case.assert().await;
}
