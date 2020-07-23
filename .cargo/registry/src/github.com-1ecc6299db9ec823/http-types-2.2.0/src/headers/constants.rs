use super::HeaderName;

/// The `Content-Encoding` Header
pub const CONTENT_ENCODING: HeaderName = HeaderName::from_lowercase_str("content-encoding");
/// The `Content-Language` Header
pub const CONTENT_LANGUAGE: HeaderName = HeaderName::from_lowercase_str("content-language");
/// The `Content-Length` Header
pub const CONTENT_LENGTH: HeaderName = HeaderName::from_lowercase_str("content-length");
/// The `Content-Location` Header
pub const CONTENT_LOCATION: HeaderName = HeaderName::from_lowercase_str("content-location");
/// The `Content-MD5` Header
pub const CONTENT_MD5: HeaderName = HeaderName::from_lowercase_str("content-md5");
/// The `Content-Range` Header
pub const CONTENT_RANGE: HeaderName = HeaderName::from_lowercase_str("content-range");
/// The `Content-Type` Header
pub const CONTENT_TYPE: HeaderName = HeaderName::from_lowercase_str("content-type");

/// The `Cookie` Header
pub const COOKIE: HeaderName = HeaderName::from_lowercase_str("cookie");

/// The `Set-Cookie` Header
pub const SET_COOKIE: HeaderName = HeaderName::from_lowercase_str("set-cookie");

/// The `Transfer-Encoding` Header
pub const TRANSFER_ENCODING: HeaderName = HeaderName::from_lowercase_str("transfer-encoding");

/// The `Date` Header
pub const DATE: HeaderName = HeaderName::from_lowercase_str("date");

/// The `Host` Header
pub const HOST: HeaderName = HeaderName::from_lowercase_str("host");

/// The `Origin` Header
pub const ORIGIN: HeaderName = HeaderName::from_lowercase_str("origin");

/// The `access-control-max-age` Header
pub const ACCESS_CONTROL_MAX_AGE: HeaderName =
    HeaderName::from_lowercase_str("access-control-max-age");
/// The `access-control-allow-origin` Header
pub const ACCESS_CONTROL_ALLOW_ORIGIN: HeaderName =
    HeaderName::from_lowercase_str("access-control-allow-origin");
/// The `access-control-allow-headers` Header
pub const ACCESS_CONTROL_ALLOW_HEADERS: HeaderName =
    HeaderName::from_lowercase_str("access-control-allow-headers");
/// The `access-control-allow-methods` Header
pub const ACCESS_CONTROL_ALLOW_METHODS: HeaderName =
    HeaderName::from_lowercase_str("access-control-allow-methods");
/// The `access-control-expose-headers` Header
pub const ACCESS_CONTROL_EXPOSE_HEADERS: HeaderName =
    HeaderName::from_lowercase_str("access-control-expose-headers");
/// The `access-control-request-method` Header
pub const ACCESS_CONTROL_REQUEST_METHOD: HeaderName =
    HeaderName::from_lowercase_str("access-control-request-method");
/// The `access-control-request-headers` Header
pub const ACCESS_CONTROL_REQUEST_HEADERS: HeaderName =
    HeaderName::from_lowercase_str("access-control-request-headers");
/// The `access-control-allow-credentials` Header
pub const ACCESS_CONTROL_ALLOW_CREDENTIALS: HeaderName =
    HeaderName::from_lowercase_str("access-control-allow-credentials");

///  The `Accept` Header
pub const ACCEPT: HeaderName = HeaderName::from_lowercase_str("accept");
///  The `Accept-Charset` Header
pub const ACCEPT_CHARSET: HeaderName = HeaderName::from_lowercase_str("accept-charset");
///  The `Accept-Encoding` Header
pub const ACCEPT_ENCODING: HeaderName = HeaderName::from_lowercase_str("accept-encoding");
///  The `Accept-Language` Header
pub const ACCEPT_LANGUAGE: HeaderName = HeaderName::from_lowercase_str("accept-language");
///  The `Accept-Ranges` Header
pub const ACCEPT_RANGES: HeaderName = HeaderName::from_lowercase_str("accept-ranges");

///  The `Age` Header
pub const AGE: HeaderName = HeaderName::from_lowercase_str("age");

///  The `Allow` Header
pub const ALLOW: HeaderName = HeaderName::from_lowercase_str("allow");

///  The `Authorization` Header
pub const AUTHORIZATION: HeaderName = HeaderName::from_lowercase_str("authorization");

///  The `Cache-Control` Header
pub const CACHE_CONTROL: HeaderName = HeaderName::from_lowercase_str("cache-control");

///  The `Connection` Header
pub const CONNECTION: HeaderName = HeaderName::from_lowercase_str("connection");

///  The `ETag` Header
pub const ETAG: HeaderName = HeaderName::from_lowercase_str("etag");

///  The `Expect` Header
pub const EXPECT: HeaderName = HeaderName::from_lowercase_str("expect");

///  The `Expires` Header
pub const EXPIRES: HeaderName = HeaderName::from_lowercase_str("expires");

///  The `From` Header
pub const FROM: HeaderName = HeaderName::from_lowercase_str("from");

///  The `If-Match` Header
pub const IF_MATCH: HeaderName = HeaderName::from_lowercase_str("if-match");

///  The `If-Modified-Since` Header
pub const IF_MODIFIED_SINCE: HeaderName = HeaderName::from_lowercase_str("if-modified-since");

///  The `If-None-Match` Header
pub const IF_NONE_MATCH: HeaderName = HeaderName::from_lowercase_str("if-none-match");

///  The `If-Range` Header
pub const IF_RANGE: HeaderName = HeaderName::from_lowercase_str("if-range");

///  The `If-Unmodified-Since` Header
pub const IF_UNMODIFIED_SINCE: HeaderName = HeaderName::from_lowercase_str("if-unmodified-since");

///  The `Last-Modified` Header
pub const LAST_MODIFIED: HeaderName = HeaderName::from_lowercase_str("last-modified");

///  The `Location` Header
pub const LOCATION: HeaderName = HeaderName::from_lowercase_str("location");

///  The `Max-Forwards` Header
pub const MAX_FORWARDS: HeaderName = HeaderName::from_lowercase_str("max-forwards");

///  The `Pragma` Header
pub const PRAGMA: HeaderName = HeaderName::from_lowercase_str("pragma");

///  The `Proxy-Authenticate` Header
pub const PROXY_AUTHENTICATE: HeaderName = HeaderName::from_lowercase_str("proxy-authenticate");
///  The `Proxy-Authorization` Header
pub const PROXY_AUTHORIZATION: HeaderName = HeaderName::from_lowercase_str("proxy-authorization");

///  The `Referer` Header
pub const REFERER: HeaderName = HeaderName::from_lowercase_str("referer");

///  The `Retry-After` Header
pub const RETRY_AFTER: HeaderName = HeaderName::from_lowercase_str("retry-after");

///  The `Server` Header
pub const SERVER: HeaderName = HeaderName::from_lowercase_str("server");

///  The `Te` Header
pub const TE: HeaderName = HeaderName::from_lowercase_str("te");

///  The `Trailer` Header
pub const TRAILER: HeaderName = HeaderName::from_lowercase_str("trailer");

///  The `Upgrade` Header
pub const UPGRADE: HeaderName = HeaderName::from_lowercase_str("upgrade");

///  The `User-Agent` Header
pub const USER_AGENT: HeaderName = HeaderName::from_lowercase_str("user-agent");

///  The `Vary` Header
pub const VARY: HeaderName = HeaderName::from_lowercase_str("vary");

///  The `Via` Header
pub const VIA: HeaderName = HeaderName::from_lowercase_str("via");

///  The `Warning` Header
pub const WARNING: HeaderName = HeaderName::from_lowercase_str("warning");

///  The `WWW-Authenticate` Header
pub const WWW_AUTHENTICATE: HeaderName = HeaderName::from_lowercase_str("www-authenticate");
