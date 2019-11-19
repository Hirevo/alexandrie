use std::net;

use diesel::prelude::*;
use http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};

pub mod database;
pub mod index;
pub mod storage;
pub mod syntect;

#[cfg(feature = "frontend")]
pub mod frontend;

use crate::db::models::Author;
use crate::db::schema::*;
use crate::index::Index;
use crate::storage::Storage;
use crate::Repo;

use crate::config::database::DatabaseConfig;
use crate::config::index::IndexConfig;
use crate::config::storage::StorageConfig;
use crate::config::syntect::{SyntectConfig, SyntectState};

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

impl State {
    /// Determines the author from the request's headers.
    pub async fn get_author(&self, headers: &HeaderMap<HeaderValue>) -> Option<Author> {
        let token = headers.get("authorization").and_then(|x| x.to_str().ok())?;

        let query = self.repo.run(|conn| {
            //? Get the author associated to this token.
            author_tokens::table
                .inner_join(authors::table)
                .select(authors::all_columns)
                .filter(author_tokens::token.eq(token))
                .first::<Author>(conn)
                .ok()
        });

        query.await
    }
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
