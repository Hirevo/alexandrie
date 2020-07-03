use serde::{Deserialize, Serialize};

/// The 'command-line' configuration.
pub mod cli;

/// The 'git2' configuration.
#[cfg(feature = "git2")]
pub mod git2;

use crate::config::cli::CommandLineIndexConfig;
use crate::Index;

#[cfg(feature = "git2")]
use crate::config::git2::Git2IndexConfig;

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
    /// The 'git2' index management strategy (uses [**`libgit2`**][libgit2]).
    /// [libgit2]: https://libgit2.org
    #[cfg(feature = "git2")]
    Git2(Git2IndexConfig),
}

impl From<IndexConfig> for Index {
    fn from(config: IndexConfig) -> Index {
        match config {
            IndexConfig::CommandLine(config) => Index::CommandLine(config.into()),
            #[cfg(feature = "git2")]
            IndexConfig::Git2(config) => Index::Git2(config.into()),
        }
    }
}
