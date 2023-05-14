use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Database configuration (`[database]` section).
pub mod database;
/// Frontend configuration (`[frontend]` section).
#[cfg(feature = "frontend")]
pub mod frontend;

/// Serde (de)serialization helper functions.
pub mod serde_utils;

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
    /// The maximum allowed crate size.
    #[serde(deserialize_with = "serde_utils::deserialize_file_size_opt")]
    max_crate_size: Option<u64>,
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

/// the general configuration state, created from [GeneralConfig].
pub struct GeneralState {
    /// The maximum crate size allowed for publication.
    pub max_crate_size: Option<u64>,
}

/// The application state, created from [Config].
pub struct State {
    /// General configuration state.
    pub general: GeneralState,
    /// The current crate indexer used.
    pub index: Index,
    /// The current crate storage strategy used.
    pub storage: Storage,
    /// The current database connection pool.
    pub db: Database,
    /// The syntect configuration.
    pub syntect: SyntectState,
    /// Search config
    pub search: RwLock<Tantivy>,
    /// The frontend configured state.
    #[cfg(feature = "frontend")]
    pub frontend: FrontendState,
}

impl From<GeneralConfig> for GeneralState {
    fn from(config: GeneralConfig) -> Self {
        Self {
            max_crate_size: config.max_crate_size,
        }
    }
}

impl TryFrom<Config> for State {
    type Error = Error;

    fn try_from(config: Config) -> Result<State, Self::Error> {
        Ok(State {
            general: config.general.into(),
            index: config.index.into(),
            storage: config.storage.into(),
            db: Database::new(&config.database),
            syntect: config.syntect.into(),
            search: RwLock::new(config.search.try_into()?),
            #[cfg(feature = "frontend")]
            frontend: config.frontend.into(),
        })
    }
}

impl State {
    /// Returns whether we require users to log in to browse crates.
    #[cfg(feature = "frontend")]
    pub fn is_login_required(&self) -> bool {
        self.frontend.config.login_required
    }
}
