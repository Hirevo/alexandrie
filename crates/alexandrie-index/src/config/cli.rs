use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::cli::CommandLineIndex;

/// The configuration struct for the 'command-line' index management strategy.
///
/// ```toml
/// [index]
/// type = "command-line" # required
/// path = "crate-index"  # required
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CommandLineIndexConfig {
    /// The path to the local index repository.
    pub path: PathBuf,
}

impl From<CommandLineIndexConfig> for CommandLineIndex {
    fn from(config: CommandLineIndexConfig) -> CommandLineIndex {
        CommandLineIndex::new(config.path)
    }
}
