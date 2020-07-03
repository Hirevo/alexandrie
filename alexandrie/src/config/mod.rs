use std::net;

use serde::{Deserialize, Serialize};

/// Database configuration (`[database]` section).
pub mod database;

/// Frontend configuration (`[frontend]` section).
#[cfg(feature = "frontend")]
pub mod frontend;

use alexandrie_index::config::IndexConfig;
use alexandrie_index::Index;
use alexandrie_rendering::config::{SyntectConfig, SyntectState};
use alexandrie_storage::config::StorageConfig;
use alexandrie_storage::Storage;

use crate::Repo;

use crate::config::database::DatabaseConfig;

#[cfg(feature = "frontend")]
pub use crate::config::frontend::*;

/// The general configuration options struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// The host address to bind on.
    pub addr: net::IpAddr,
    /// The port to listen on.
    pub port: u16,
}

/// The application configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// General instance configuration options.
    pub general: GeneralConfig,
    /// The crate index management strategy to use.
    pub index: IndexConfig,
    /// The crate storage strategy to use.
    pub storage: StorageConfig,
    /// The database configuration.
    pub database: DatabaseConfig,
    /// The syntax-highlighting configuration.
    pub syntect: SyntectConfig,
    /// The frontend configuration.
    #[cfg(feature = "frontend")]
    pub frontend: FrontendConfig,
}

/// The application state, created from [Config].
pub struct State {
    /// The current crate indexer used.
    pub index: Index,
    /// The current crate storage strategy used.
    pub storage: Storage,
    /// The current database connection pool.
    pub repo: Repo,
    /// The syntect configuration.
    pub syntect: SyntectState,
    /// The frontend configured state.
    #[cfg(feature = "frontend")]
    pub frontend: FrontendState,
}

impl From<Config> for State {
    fn from(config: Config) -> State {
        State {
            index: config.index.into(),
            storage: config.storage.into(),
            repo: Repo::new(&config.database),
            syntect: config.syntect.into(),
            #[cfg(feature = "frontend")]
            frontend: config.frontend.into(),
        }
    }
}
