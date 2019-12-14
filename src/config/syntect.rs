use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use syntect::dumps;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;

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

/// The syntax-highlighting state struct, created from [SyntectConfig].
pub struct SyntectState {
    /// The loaded syntax set.
    pub syntaxes: SyntaxSet,
    /// The loaded theme set.
    pub themes: ThemeSet,
    /// The chosen theme's name.
    pub theme_name: String,
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
