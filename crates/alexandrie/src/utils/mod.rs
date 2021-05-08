/// Various authentication-related utilities.
pub mod auth;
/// Build time debug information.
pub mod build;
/// Various utilities to check for common properties.
pub mod checks;
/// Utilities to issue logs about requests.
pub mod request_log;
/// Various utilities to assist building HTTP responses.
pub mod response;
/// Various session-related utilities.
pub mod sessions;

/// Transforms a crate name to its canonical form.
///
/// A canonical crate name means that all dashes ('-') have
/// been replaced by underscores ('_') for consistency, because
/// they are considered to be equivalent.
pub fn canonical_name(name: impl AsRef<str>) -> String {
    name.as_ref().to_ascii_lowercase().replace("-", "_")
}
