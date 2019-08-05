use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

#[cfg(feature = "frontend")]
use handlebars::Handlebars;

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
    #[serde(default = "enabled_def")]
    pub enabled: bool,
    pub title: Option<String>,
    pub description: Option<String>,
    pub favicon: Option<String>,
}

/// The database configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyntectThemesConfig {
    Dump { path: PathBuf },
    Directory { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum SyntectSyntaxesConfig {
    Dump { path: PathBuf },
    Directory { path: PathBuf },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SyntectConfig {
    pub themes: SyntectThemesConfig,
    pub syntaxes: SyntectSyntaxesConfig,
}

/// The application configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Config {
    /// The crate index management strategy to use.
    pub index: Index,
    /// The crate storage strategy to use.
    pub storage: Storage,
    /// The database configuration.
    pub database: DatabaseConfig,
    /// The syntect configuration.
    pub syntect: SyntectConfig,
    /// The frontend configuration.
    #[cfg(feature = "frontend")]
    pub frontend: FrontendConfig,
}

pub struct SyntectState {
    pub syntaxes: SyntaxSet,
    pub themes: ThemeSet,
}

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
