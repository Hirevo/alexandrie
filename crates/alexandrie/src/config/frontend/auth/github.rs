use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use url::Url;
use serde::{Deserialize, Serialize};

use crate::error::Error;


/// The configuration struct for the "github" authentication strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GithubAuthConfig {
    /// Whether the strategy is enabled.
    pub enabled: bool,
    /// The client ID for the OAuth2 application as provided by GitHub.
    pub client_id: String,
    /// The client secret for the OAuth2 application as provided by GitHub.
    pub client_secret: String,
    /// The list of organizations for which membership of one of them is mandated.
    pub allowed_organizations: Option<Vec<GithubAuthOrganizationConfig>>,
    /// Whether creating a new account is allowed using this strategy.
    pub allow_registration: bool,
}

/// The configuration struct for an an allowed organization for the "github" authentication strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GithubAuthOrganizationConfig {
    /// The name of the organization.
    pub name: String,
    /// The list of teams for which membership of one of them is mandated.
    pub allowed_teams: Option<Vec<String>>,
}

impl Default for GithubAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            client_id: String::default(),
            client_secret: String::default(),
            allowed_organizations: None,
            allow_registration: true,
        }
    }
}


/// The authentication state for the "github" strategy.
pub struct GithubAuthState {
    /// The OAuth2 client configured for the "github" strategy.
    pub client: BasicClient,
}

impl GithubAuthState {
    /// Create a new [`GithubAuthState`] from a [`GithubAuthConfig`] and the origin of the current Alexandrie instance.
    pub fn new(config: &GithubAuthConfig, origin: &str) -> Result<Self, Error> {
        let mut redirect_url = Url::parse(&origin)?;
        redirect_url.set_path("/account/github/callback");

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
            Some(TokenUrl::new("https://github.com/login/oauth/access_token".to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::from_url(redirect_url));

        Ok(Self { client })
    }
}
