use std::path::PathBuf;

use serde::{Deserialize, Serialize};

fn enabled_def() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "enabled_def")]
    pub enabled: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub favicon: Option<PathBuf>,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            enabled: true,
            title: Some(String::from("Alexandrie")),
            description: Some(String::from(
                "An alternative crate registry for Cargo, the Rust package manager.",
            )),
            favicon: None,
        }
    }
}
