use std::net;
use std::path::PathBuf;

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
use tide::cookies::ContextExt;
use tide::http::{HeaderMap, HeaderValue};

#[cfg(feature = "frontend")]
use handlebars::Handlebars;

use crate::db::models::Author;
use crate::db::schema::*;
use crate::index::Index;
use crate::storage::Storage;
use crate::Repo;

fn enabled_def() -> bool {
    true
}

#[cfg(feature = "frontend")]
/// The frontend configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// Is the frontend enabled ?
    #[serde(default = "enabled_def")]
    pub enabled: bool,
    /// The instance's title.
    pub title: Option<String>,
    /// The instance's description.
    pub description: Option<String>,
    /// The path to the instance's favicon.
    pub favicon: Option<String>,
}

/// The database configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
}

/// The syntax-highlighting theme configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyntectThemesConfig {
    Dump { path: PathBuf },
    Directory { path: PathBuf },
}

/// The syntax-highlighting syntaxes configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyntectSyntaxesConfig {
    Dump { path: PathBuf },
    Directory { path: PathBuf },
}

/// The complete syntax-highlighting configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntectConfig {
    pub themes: SyntectThemesConfig,
    pub syntaxes: SyntectSyntaxesConfig,
}

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
    pub index: Index,
    /// The crate storage strategy to use.
    pub storage: Storage,
    /// The database configuration.
    pub database: DatabaseConfig,
    /// The syntax-highlighting configuration.
    pub syntect: SyntectConfig,
    /// The frontend configuration.
    #[cfg(feature = "frontend")]
    pub frontend: FrontendConfig,
}

/// The syntax-highlighting state struct, created from [SyntectConfig].
pub struct SyntectState {
    pub syntaxes: SyntaxSet,
    pub themes: ThemeSet,
}

/// The frontend state struct, created from [FrontendConfig].
#[cfg(feature = "frontend")]
pub struct FrontendState {
    pub handlebars: Handlebars,
    pub config: FrontendConfig,
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
        let token = headers.get("Authorization").and_then(|x| x.to_str().ok())?;

        let query = self.repo.run(|conn| {
            author_tokens::table
                .inner_join(authors::table)
                .select(authors::all_columns)
                .filter(author_tokens::token.eq(token))
                .first::<Author>(&conn)
                .ok()
        });

        query.await
    }
}

impl From<Config> for State {
    fn from(config: Config) -> State {
        State {
            index: config.index,
            storage: config.storage,
            repo: Repo::new(config.database.url.as_str()),
            syntect: config.syntect.into(),
            #[cfg(feature = "frontend")]
            frontend: config.frontend.into(),
        }
    }
}

impl From<SyntectConfig> for SyntectState {
    fn from(config: SyntectConfig) -> SyntectState {
        SyntectState {
            syntaxes: match config.syntaxes {
                SyntectSyntaxesConfig::Dump { path } => {
                    dumps::from_dump_file(&path).expect("couldn't load syntaxes' dump file")
                }
                SyntectSyntaxesConfig::Directory { path } => SyntaxSet::load_from_folder(&path)
                    .expect("couldn't load syntaxes from directory"),
            },
            themes: match config.themes {
                SyntectThemesConfig::Dump { path } => {
                    dumps::from_dump_file(&path).expect("couldn't load themes' dump")
                }
                SyntectThemesConfig::Directory { path } => {
                    ThemeSet::load_from_folder(&path).expect("couldn't load themes from directory")
                }
            },
        }
    }
}

#[cfg(feature = "frontend")]
impl From<FrontendConfig> for FrontendState {
    fn from(config: FrontendConfig) -> FrontendState {
        FrontendState {
            config,
            handlebars: {
                let mut engine = Handlebars::new();
                engine
                    .register_templates_directory(".hbs", "templates")
                    .expect("handlebars configuration error");
                engine
            },
        }
    }
}
