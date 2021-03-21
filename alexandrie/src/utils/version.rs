/// add mod include build with shadow-rs script
pub mod shadow {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

///build const value and version with shadow-rs script
pub fn version() -> String {
    format!(
        "version:{}\nbranch:{}\ncommit-hash:{}\nbuild_time:{}\nbuild_env:{},{}",
        shadow::PKG_VERSION,
        shadow::BRANCH,
        shadow::SHORT_COMMIT,
        shadow::BUILD_TIME,
        shadow::RUST_VERSION,
        shadow::RUST_CHANNEL,
    )
}
