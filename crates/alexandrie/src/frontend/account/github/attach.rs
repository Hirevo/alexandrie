use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use axum_sessions::extractors::WritableSession;
use oauth2::{CsrfToken, Scope};

use crate::config::AppState;
use crate::error::FrontendError;
use crate::frontend::account::github::{GithubLoginState, GITHUB_LOGIN_STATE_KEY};
use crate::utils;
use crate::utils::auth::frontend::Auth;

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Auth>,
    mut session: WritableSession,
) -> Result<Either<(StatusCode, Html<String>), Redirect>, FrontendError> {
    let Some(Auth(author)) = maybe_author else {
        return Ok(Either::E2(Redirect::to("/account/manage")));
    };

    let github_config = &state.frontend.config.auth.github;
    let Some(github_state) = state.frontend.auth.github.as_ref() else {
        let rendered = utils::response::error_html(
            state.as_ref(),
            Some(author),
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

    let (url, state) = builder.url();

    let data = GithubLoginState {
        state,
        attach: true,
    };
    session.insert(GITHUB_LOGIN_STATE_KEY, &data)?;

    return Ok(Either::E2(Redirect::to(url.as_str())));
}
