pub mod shadow {
    include!(concat!(env!("OUT_DIR"), "/shadow.rs"));
}

pub fn version() -> String {
    format!(
        "branch:{}\ncommit-hash:{}\nbuild_time:{}\nbuild_env:{},{}",
        shadow::BRANCH,
        shadow::SHORT_COMMIT,
        shadow::BUILD_TIME,
        shadow::RUST_VERSION,
        shadow::RUST_CHANNEL,
    )
}
