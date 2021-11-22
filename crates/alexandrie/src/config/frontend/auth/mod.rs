use serde::{Serialize, Deserialize};

/// Types for the "github" authentication strategy.
pub mod github;
/// Types for the "gitlab" authentication strategy.
pub mod gitlab;
/// Types for the "local" authentication strategy.
pub mod local;

use crate::config::frontend::auth::github::{GithubAuthState, GithubAuthConfig};
use crate::config::frontend::auth::gitlab::{GitlabAuthState, GitlabAuthConfig};
use crate::config::frontend::auth::local::{LocalAuthState, LocalAuthConfig};
use crate::error::Error;

/// The configuration struct for authentication strategies.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthConfig {
    /// The origin at which this current instance of Alexandrie is reachable.
    pub origin: String,
    /// Configuration regarding the "local" authentication strategy.
    #[serde(default)]
    pub local: LocalAuthConfig,
    /// Configuration regarding the "github" authentication strategy.
    #[serde(default)]
    pub github: GithubAuthConfig,
    /// Configuration regarding the "gitlab" authentication strategy.
    #[serde(default)]
    pub gitlab: GitlabAuthConfig,
}


/// The authentication state, having things like OAuth clients for external authentication.
pub struct AuthState {
    /// The authentication state for the "local" strategy, if enabled.
    pub local: Option<LocalAuthState>,
    /// The authentication state for the "github" strategy, if enabled.
    pub github: Option<GithubAuthState>,
    /// The authentication state for the "gitlab" strategy, if enabled.
    pub gitlab: Option<GitlabAuthState>,
}

impl AuthState {
    /// Create a new [`AuthState`] from an [`AuthConfig`].
    pub fn new(config: &AuthConfig) -> Result<Self, Error> {
        let local = (config.local.enabled)
            .then(|| LocalAuthState::new(&config.local))
            .transpose()?;
        let github = (config.github.enabled)
            .then(|| GithubAuthState::new(&config.github, &config.origin))
            .transpose()?;
        let gitlab = (config.gitlab.enabled)
            .then(|| GitlabAuthState::new(&config.gitlab, &config.origin))
            .transpose()?;

        Ok(Self {
            local,
            github,
            gitlab,
        })
    }
}
