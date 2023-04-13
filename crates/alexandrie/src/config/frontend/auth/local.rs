use serde::{Deserialize, Serialize};

use crate::error::Error;

/// The configuration struct for the "local" authentication strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalAuthConfig {
    /// Whether the strategy is enabled.
    pub enabled: bool,
    /// Whether creating a new account is allowed using this strategy.
    pub allow_registration: bool,
}

impl Default for LocalAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            allow_registration: true,
        }
    }
}

/// The authentication state for the "local" strategy.
pub struct LocalAuthState {}

impl LocalAuthState {
    /// Create a new [`LocalAuthState`] from a [`LocalAuthConfig`].
    pub fn new(_: &LocalAuthConfig) -> Result<Self, Error> {
        Ok(Self {})
    }
}
