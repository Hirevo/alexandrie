//! HTTP Security Headers.
//!
//! ## Example
//!
//! ```
//! use http_types::{StatusCode, Response};
//!
//! let mut res = Response::new(StatusCode::Ok);
//! http_types::security::default(&mut res);
// //! assert_eq!(res["X-Content-Type-Options"], "nosniff");
// //! assert_eq!(res["X-XSS-Protection"], "1; mode=block");
//! ```

use crate::headers::{HeaderName, HeaderValue, Headers};
pub use csp::{ContentSecurityPolicy, ReportTo, ReportToEndpoint, Source};

mod csp;

/// Apply a set of default protections.
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// http_types::security::default(&mut headers);
// /// assert_eq!(headers["X-Content-Type-Options"], "nosniff");
// /// assert_eq!(headers["X-XSS-Protection"], "1; mode=block");
// /// ```
pub fn default(mut headers: impl AsMut<Headers>) {
    dns_prefetch_control(&mut headers);
    nosniff(&mut headers);
    frameguard(&mut headers, None);
    powered_by(&mut headers, None);
    hsts(&mut headers);
    xss_filter(&mut headers);
}

/// Disable browsers’ DNS prefetching by setting the `X-DNS-Prefetch-Control` header.
///
/// [read more](https://helmetjs.github.io/docs/dns-prefetch-control/)
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// http_types::security::dns_prefetch_control(&mut headers);
// /// assert_eq!(headers["X-DNS-Prefetch-Control"], "on");
// /// ```
#[inline]
pub fn dns_prefetch_control(mut headers: impl AsMut<Headers>) {
    headers.as_mut().insert("X-DNS-Prefetch-Control", "on");
}

/// Set the frameguard level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrameOptions {
    /// Set to `sameorigin`
    SameOrigin,
    /// Set to `deny`
    Deny,
}

/// Mitigates clickjacking attacks by setting the `X-Frame-Options` header.
///
/// [read more](https://helmetjs.github.io/docs/frameguard/)
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// http_types::security::frameguard(&mut headers, None);
// /// assert_eq!(headers["X-Frame-Options"], "sameorigin");
// /// ```
#[inline]
pub fn frameguard(mut headers: impl AsMut<Headers>, guard: Option<FrameOptions>) {
    let kind = match guard {
        None | Some(FrameOptions::SameOrigin) => "sameorigin",
        Some(FrameOptions::Deny) => "deny",
    };
    headers.as_mut().insert("X-Frame-Options", kind);
}

/// Removes the `X-Powered-By` header to make it slightly harder for attackers to see what
/// potentially-vulnerable technology powers your site.
///
/// [read more](https://helmetjs.github.io/docs/hide-powered-by/)
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// headers.as_mut().insert("X-Powered-By", "Tide/Rust".parse());
// /// http_types::security::hide_powered_by(&mut headers);
// /// assert_eq!(headers.get("X-Powered-By"), None);
// /// ```
#[inline]
pub fn powered_by(mut headers: impl AsMut<Headers>, value: Option<HeaderValue>) {
    let name = HeaderName::from_lowercase_str("X-Powered-By");
    match value {
        Some(value) => {
            headers.as_mut().insert(name, value);
        }
        None => {
            headers.as_mut().remove(name);
        }
    };
}

/// Sets the `Strict-Transport-Security` header to keep your users on `HTTPS`.
///
/// Note that the header won’t tell users on HTTP to switch to HTTPS, it will tell HTTPS users to
/// stick around. Defaults to 60 days.
///
/// [read more](https://helmetjs.github.io/docs/hsts/)
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// http_types::security::hsts(&mut headers);
// /// assert_eq!(headers["Strict-Transport-Security"], "max-age=5184000");
// /// ```
#[inline]
pub fn hsts(mut headers: impl AsMut<Headers>) {
    headers
        .as_mut()
        .insert("Strict-Transport-Security", "max-age=5184000");
}

/// Prevent browsers from trying to guess (“sniff”) the MIME type, which can have security
/// implications.
///
/// [read more](https://helmetjs.github.io/docs/dont-sniff-mimetype/)
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// http_types::security::nosniff(&mut headers);
// /// assert_eq!(headers["X-Content-Type-Options"], "nosniff");
// /// ```
#[inline]
pub fn nosniff(mut headers: impl AsMut<Headers>) {
    headers.as_mut().insert("X-Content-Type-Options", "nosniff");
}

/// Sets the `X-XSS-Protection` header to prevent reflected XSS attacks.
///
/// [read more](https://helmetjs.github.io/docs/xss-filter/)
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// http_types::security::xss_filter(&mut headers);
// /// assert_eq!(headers["X-XSS-Protection"], "1; mode=block");
// /// ```
#[inline]
pub fn xss_filter(mut headers: impl AsMut<Headers>) {
    headers.as_mut().insert("X-XSS-Protection", "1; mode=block");
}

/// Set the Referrer-Policy level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ReferrerOptions {
    /// Set to "no-referrer"
    NoReferrer,
    /// Set to "no-referrer-when-downgrade" the default
    NoReferrerDowngrade,
    /// Set to "same-origin"
    SameOrigin,
    /// Set to "origin"
    Origin,
    /// Set to "strict-origin"
    StrictOrigin,
    /// Set to "origin-when-cross-origin"
    CrossOrigin,
    /// Set to "strict-origin-when-cross-origin"
    StrictCrossOrigin,
    /// Set to "unsafe-url"
    UnsafeUrl,
}

/// Mitigates referrer leakage by controlling the referer[sic] header in links away from pages
///
/// [read more](https://scotthelme.co.uk/a-new-security-header-referrer-policy/)
///
/// [Mozilla Developer Network](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Referrer-Policy)
///
///
// /// ## Examples
// /// ```
// /// use http_types::Response;
// ///
// /// let mut res = Response::new(StatusCode::Ok);
// /// http_types::security::referrer_policy(&mut headers, Some(http_types::security::ReferrerOptions::UnsafeUrl));
// /// http_types::security::referrer_policy(&mut headers, None);
// /// let mut referrerValues: Vec<&str> = headers.get_all("Referrer-Policy").iter().map(|x| x.to_str().unwrap()).collect();
// /// assert_eq!(referrerValues.sort(), vec!("unsafe-url", "no-referrer").sort());
// /// ```
#[inline]
pub fn referrer_policy(mut headers: impl AsMut<Headers>, referrer: Option<ReferrerOptions>) {
    let policy = match referrer {
        None | Some(ReferrerOptions::NoReferrer) => "no-referrer",
        Some(ReferrerOptions::NoReferrerDowngrade) => "no-referrer-when-downgrade",
        Some(ReferrerOptions::SameOrigin) => "same-origin",
        Some(ReferrerOptions::Origin) => "origin",
        Some(ReferrerOptions::StrictOrigin) => "strict-origin",
        Some(ReferrerOptions::CrossOrigin) => "origin-when-cross-origin",
        Some(ReferrerOptions::StrictCrossOrigin) => "strict-origin-when-cross-origin",
        Some(ReferrerOptions::UnsafeUrl) => "unsafe-url",
    };

    // We MUST allow for multiple Referrer-Policy headers to be set.
    // See: https://w3c.github.io/webappsec-referrer-policy/#unknown-policy-values example #13
    headers.as_mut().append("Referrer-Policy", policy);
}
