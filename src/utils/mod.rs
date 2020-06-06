/// Various authentication-related utilities.
pub mod auth;
/// Various utilities to check for common properties.
pub mod checks;
/// Various utilities to assist Markdown rendering to HTML.
pub mod rendering;
/// Utilities to issue logs about requests.
pub mod request_log;
/// Various utilities to assist building HTTP responses.
pub mod response;

/// Utilities for using cookies.
#[cfg(feature = "frontend")]
pub mod cookies;
/// Utilities for using flash cookies.
#[cfg(feature = "frontend")]
pub mod flash;
