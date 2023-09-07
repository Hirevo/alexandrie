use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use axum_sessions::extractors::WritableSession;
use oauth2::{CsrfToken, Scope};
use serde::{Deserialize, Serialize};

/// Endpoint to attach to an existing Alexandrie account.
pub mod attach;
/// Callback endpoint for the "github" authentication strategy.
pub mod callback;
/// Endpoint to detach from an existing Alexandrie account.
pub mod detach;

use crate::config::AppState;
use crate::error::FrontendError;
use crate::utils;
use crate::utils::auth::frontend::Auth;
use crate::utils::response::common;

const GITHUB_LOGIN_STATE_KEY: &str = "login.github";

#[derive(Clone, Serialize, Deserialize)]
struct GithubLoginState {
    state: CsrfToken,
    attach: bool,
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Auth>,
    mut session: WritableSession,
) -> Result<Either<(StatusCode, Html<String>), Redirect>, FrontendError> {
    if let Some(Auth(author)) = maybe_author {
        let state = state.as_ref();
        let response = common::already_logged_in(state, author)?;
        return Ok(Either::E1(response));
    }

    let github_config = &state.frontend.config.auth.github;
    let Some(github_state) = state.frontend.auth.github.as_ref() else {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "authentication using GitHub is not allowed on this instance",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
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

    let data = GithubLoginState {
        state,
        attach: false,
    };
    session.insert(GITHUB_LOGIN_STATE_KEY, &data)?;

    return Ok(Either::E2(Redirect::to(url.as_str())));
}
