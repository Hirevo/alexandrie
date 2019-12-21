/// Various utilities to check for common properties.
pub mod checks;
/// Various utilities to assist Markdown rendering to HTML.
pub mod rendering;
/// Utilities to issue logs about requests.
pub mod request_log;
/// Various utilities to assist building HTTP responses.
pub mod response;
/// A HTTP handler for serving static files.
pub mod static_files;

/// Various authentication-related utilities.
#[cfg(feature = "frontend")]
pub mod auth;
/// Utilities for using cookies.
#[cfg(feature = "frontend")]
pub mod cookies;
/// Utilities for using flash cookies.
#[cfg(feature = "frontend")]
pub mod flash;
