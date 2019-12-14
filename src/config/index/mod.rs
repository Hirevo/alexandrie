use serde::{Deserialize, Serialize};

/// The 'command-line' configuration.
pub mod cli;

use crate::config::index::cli::CommandLineIndexConfig;
use crate::index::Index;

/// The configuration enum for index management strategies.
///
/// ```toml
/// [index]
/// type = "<...>" # required, replace "<...>" by the selected strategy.
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum IndexConfig {
    /// The 'command-line' index management strategy (uses "git" shell command).
    CommandLine(CommandLineIndexConfig),
}

impl From<IndexConfig> for Index {
    fn from(config: IndexConfig) -> Index {
        match config {
            IndexConfig::CommandLine(config) => Index::CommandLine(config.into()),
        }
    }
}
