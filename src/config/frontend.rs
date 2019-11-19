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

/// The frontend state struct, created from [FrontendConfig].
pub struct FrontendState {
    /// The Handlebars rendering struct.
    pub handlebars: Handlebars,
    /// The frontend configuration.
    pub config: FrontendConfig,
}

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
