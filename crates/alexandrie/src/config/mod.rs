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

use crate::db::Database;

#[cfg(feature = "frontend")]
pub use crate::config::frontend::*;
use crate::error::Error;
use crate::fts::Tantivy;

use self::database::DatabaseConfig;

/// The general configuration options struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// The address to bind the server on.
    pub bind_address: String,
}

/// Configuration for search index.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchConfig {
    /// Path to the directory where Tantivy will store its index.
    pub directory: String,
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
    /// Search config
    pub search: SearchConfig,
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
    pub db: Database,
    /// The syntect configuration.
    pub syntect: SyntectState,
    /// Search config
    pub search: Tantivy,
    /// The frontend configured state.
    #[cfg(feature = "frontend")]
    pub frontend: FrontendState,
}

impl TryFrom<Config> for State {
    type Error = Error;

    fn try_from(config: Config) -> Result<State, Self::Error> {
        Ok(State {
            index: config.index.into(),
            storage: config.storage.into(),
            db: Database::new(&config.database),
            syntect: config.syntect.into(),
            search: config.search.try_into()?,
            #[cfg(feature = "frontend")]
            frontend: config.frontend.into(),
        })
    }
}
