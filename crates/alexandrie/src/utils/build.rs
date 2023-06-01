/// Build configuration information.
pub mod built {
    // The file has been placed there by the build script.
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Formats a short build information string.
pub fn short() -> String {
    format!(
        "{version} - {commit} ({features})",
        version = built::PKG_VERSION,
        commit = built::GIT_COMMIT_HASH_SHORT.unwrap(),
        features = built::FEATURES_STR,
    )
}

/// Formats a more complete build information string.
pub fn long() -> String {
    format!(
        "version = {version}\n\
        branch = {branch}\n\
        commit = {commit}\n\
        features = {features}\n\
        build time (UTC) = {build_time}\n\
        rustc version = {rustc_version}",
        version = built::PKG_VERSION,
        branch = built::GIT_HEAD_REF.unwrap_or("(detached HEAD)"),
        commit = built::GIT_COMMIT_HASH.unwrap(),
        features = built::FEATURES_STR,
        build_time = built::BUILT_TIME_UTC,
        rustc_version = built::RUSTC_VERSION,
    )
}

/// Build time in local timezone.
pub fn built_time() -> chrono::DateTime<chrono::Local> {
    chrono::DateTime::parse_from_rfc2822(built::BUILT_TIME_UTC)
        .unwrap()
        .with_timezone(&chrono::offset::Local)
}
