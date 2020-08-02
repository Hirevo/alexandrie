use handlebars::Handlebars;
use serde::{Deserialize, Serialize};

fn enabled_def() -> bool {
    true
}

/// Represent a link entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Link {
    /// The name of the related link.
    pub name: String,
    /// The target of the related link.
    pub href: String,
}

/// The asset configuration options struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetsConfig {
    /// Assets directory path.
    pub path: String,
}

/// The templates configuration options struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TemplatesConfig {
    /// Templates directory path.
    pub path: String,
}

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
    /// Assets configuration options.
    pub assets: AssetsConfig,
    /// Templates configuration options.
    pub templates: TemplatesConfig,
}

/// The frontend state struct, created from [FrontendConfig].
pub struct FrontendState {
    /// The Handlebars rendering struct.
    pub handlebars: Handlebars<'static>,
    /// The frontend configuration.
    pub config: FrontendConfig,
}

impl From<FrontendConfig> for FrontendState {
    fn from(config: FrontendConfig) -> FrontendState {
        let mut engine = Handlebars::new();
        engine
            .register_templates_directory(".hbs", &config.templates.path)
            .expect("handlebars configuration error");

        FrontendState {
            config,
            handlebars: { engine },
        }
    }
}
