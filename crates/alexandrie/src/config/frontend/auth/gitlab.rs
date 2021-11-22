use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, RevocationUrl, TokenUrl};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::error::Error;

/// The configuration struct for the "gitlab" authentication strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitlabAuthConfig {
    /// Whether the strategy is enabled.
    pub enabled: bool,
    /// The origin at which the remote GitLab instance is reachable.
    pub origin: Url,
    /// The client ID for the OAuth2 application as provided by GitHub.
    pub client_id: String,
    /// The client secret for the OAuth2 application as provided by GitHub.
    pub client_secret: String,
    /// The list of allowed groups for authors to be authorized.
    pub allowed_groups: Option<Vec<String>>,
    /// Whether creating a new account is allowed using this strategy.
    pub allow_registration: bool,
}

impl Default for GitlabAuthConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            origin: Url::parse("https://gitlab.com").unwrap(),
            client_id: String::default(),
            client_secret: String::default(),
            allowed_groups: None,
            allow_registration: true,
        }
    }
}

/// The authentication state for the "gitlab" strategy.
pub struct GitlabAuthState {
    /// The OAuth2 client configured for the "gitlab" strategy.
    pub client: BasicClient,
}

impl GitlabAuthState {
    /// Create a new [`GitlabAuthState`] from a [`GitlabAuthConfig`] and the origin of the current Alexandrie instance.
    pub fn new(config: &GitlabAuthConfig, origin: &str) -> Result<Self, Error> {
        let mut auth_url = config.origin.clone();
        auth_url.set_path("/oauth/authorize");

        let mut token_url = config.origin.clone();
        token_url.set_path("/oauth/token");

        let mut redirect_url = Url::parse(&origin)?;
        redirect_url.set_path("/account/gitlab/callback");

        let mut revocation_url = config.origin.clone();
        revocation_url.set_path("/oauth/revoke");

        let client = BasicClient::new(
            ClientId::new(config.client_id.clone()),
            Some(ClientSecret::new(config.client_secret.clone())),
            AuthUrl::from_url(auth_url),
            Some(TokenUrl::from_url(token_url)),
        )
        .set_redirect_uri(RedirectUrl::from_url(redirect_url))
        .set_revocation_uri(RevocationUrl::from_url(revocation_url));

        Ok(Self { client })
    }
}
