use oauth2::{CsrfToken, Scope};
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

/// Callback endpoint for the "github" authentication strategy.
pub mod callback;
/// Endpoint to attach to an existing Alexandrie account.
pub mod attach;
/// Endpoint to detach from an existing Alexandrie account.
pub mod detach;

use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::response::common;
use crate::State;

const GITHUB_LOGIN_STATE_KEY: &str = "login.github";

#[derive(Clone, Serialize, Deserialize)]
struct GithubLoginState {
    state: CsrfToken,
    attach: bool,
}

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    if let Some(author) = req.get_author() {
        let state = req.state().as_ref();
        return common::already_logged_in(state, author);
    }

    let github_config = &req.state().frontend.config.auth.github;
    let github_state = match req.state().frontend.auth.github.as_ref() {
        Some(state) => state,
        None => {
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::BadRequest,
                "authentication using GitHub is not allowed on this instance",
            );
        }
    };

    let mut builder = github_state
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read:user".to_string()))
        .add_scope(Scope::new("user:email".to_string()));

    if github_config.allowed_organizations.is_some() {
        builder = builder.add_scope(Scope::new("read:org".to_string()));
    }

    let (url, state) = builder
        .add_extra_param("allow_signup", github_config.allow_registration.to_string())
        .url();

    let data = GithubLoginState { state, attach: false };
    req.session_mut().insert(GITHUB_LOGIN_STATE_KEY, &data)?;

    return Ok(utils::response::redirect(url.as_str()));
}
