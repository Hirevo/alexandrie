use std::net;
use std::path::PathBuf;

use diesel::prelude::*;
use http::{HeaderMap, HeaderValue};
use serde::{Deserialize, Serialize};
use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[cfg(feature = "frontend")]
use handlebars::Handlebars;

use crate::db::models::Author;
use crate::db::schema::*;
use crate::index::Index;
use crate::storage::Storage;
use crate::Repo;

#[cfg(feature = "frontend")]
fn enabled_def() -> bool {
    true
}

#[cfg(feature = "frontend")]
/// Represent a link entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The name of the related link.
    pub name: String,
    /// The target of the related link.
    pub href: String,
}

#[cfg(feature = "frontend")]
/// The frontend configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FrontendConfig {
    /// Is the frontend enabled?
    #[serde(default = "enabled_def")]
    pub enabled: bool,
    /// The instance's title.
    pub title: Option<String>,
    /// The instance's description.
    pub description: Option<String>,
    /// The path to the instance's favicon.
    pub favicon: Option<String>,
    /// Some related links.
    pub links: Option<Vec<Link>>,
}

/// The database configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
    /// Max connection pool size
    pub connection_pool_max_size: Option<u32>,
}

/// The syntax-highlighting themes configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyntectThemesConfig {
    /// Variant for loading themes from a binary dump.
    Dump {
        /// The path to the binary dump to load themes from.
        path: PathBuf,
        /// The name of the theme to use.
        theme_name: String,
    },
    /// Variant for recursively loading themes from a directory.
    Directory {
        /// The path to the directory to load themes from.
        path: PathBuf,
        /// The name of the theme to use.
        theme_name: String,
    },
}

/// The syntax-highlighting syntaxes configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyntectSyntaxesConfig {
    /// Variant for loading syntaxes from a binary dump.
    Dump {
        /// The path to the binary dump to load syntaxes from.
        path: PathBuf,
    },
    /// Variant for recursively loading syntaxes from a directory.
    Directory {
        /// The path to the directory to load syntaxes from.
        path: PathBuf,
    },
}

/// The complete syntax-highlighting configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntectConfig {
    /// The highlighting themes configuration.
    pub themes: SyntectThemesConfig,
    /// The highlighting syntaxes configuration.
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
    /// The loaded syntax set.
    pub syntaxes: SyntaxSet,
    /// The loaded theme set.
    pub themes: ThemeSet,
    /// The chosen theme's name.
    pub theme_name: String,
}

/// The frontend state struct, created from [FrontendConfig].
#[cfg(feature = "frontend")]
pub struct FrontendState {
    /// The Handlebars rendering struct.
    pub handlebars: Handlebars,
    /// The frontend configuration.
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
            index: config.index,
            storage: config.storage,
            repo: Repo::new(&config.database),
            syntect: config.syntect.into(),
            #[cfg(feature = "frontend")]
            frontend: config.frontend.into(),
        }
    }
}

impl From<SyntectConfig> for SyntectState {
    fn from(config: SyntectConfig) -> SyntectState {
        let syntaxes = match config.syntaxes {
            SyntectSyntaxesConfig::Dump { path } => {
                dumps::from_dump_file(&path).expect("couldn't load syntaxes' dump file")
            }
            SyntectSyntaxesConfig::Directory { path } => {
                SyntaxSet::load_from_folder(&path).expect("couldn't load syntaxes from directory")
            }
        };
        let (themes, theme_name) = match config.themes {
            SyntectThemesConfig::Dump { path, theme_name } => (
                dumps::from_dump_file(&path).expect("couldn't load themes' dump"),
                theme_name,
            ),
            SyntectThemesConfig::Directory { path, theme_name } => (
                ThemeSet::load_from_folder(&path).expect("couldn't load themes from directory"),
                theme_name,
            ),
        };
        if !themes.themes.contains_key(theme_name.as_str()) {
            panic!("no theme named '{0}' has been found", theme_name);
        }
        SyntectState {
            syntaxes,
            themes,
            theme_name,
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
