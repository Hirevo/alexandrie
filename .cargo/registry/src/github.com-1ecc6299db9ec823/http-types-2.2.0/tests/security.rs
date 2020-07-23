use http_types::{security, Response, StatusCode};

#[test]
fn security_test() {
    let mut policy = security::ContentSecurityPolicy::new();
    policy
        .default_src(security::Source::SameOrigin)
        .default_src("areweasyncyet.rs")
        .script_src(security::Source::SameOrigin)
        .script_src(security::Source::UnsafeInline)
        .object_src(security::Source::None)
        .base_uri(security::Source::None)
        .upgrade_insecure_requests();

    let mut res = Response::new(StatusCode::Ok);
    res.set_body("Hello, Chashu!");

    security::default(&mut res);
    policy.apply(&mut res);

    assert_eq!(res["content-security-policy"], "base-uri 'none'; default-src 'self' areweasyncyet.rs; object-src 'none'; script-src 'self' 'unsafe-inline'; upgrade-insecure-requests");
}
