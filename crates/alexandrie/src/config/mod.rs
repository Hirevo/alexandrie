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

use self::database::DatabaseConfig;

/// The general configuration options struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// The address to bind the server on.
    pub bind_address: String,
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
    pub db: Database,
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
            db: Database::new(&config.database),
            syntect: config.syntect.into(),
            #[cfg(feature = "frontend")]
            frontend: config.frontend.into(),
        }
    }
}
