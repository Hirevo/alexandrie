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
/// Callback endpoint for the "gitlab" authentication strategy.
pub mod callback;
/// Endpoint to detach from an existing Alexandrie account.
pub mod detach;

use crate::config::AppState;
use crate::db::models::Author;
use crate::error::FrontendError;
use crate::utils;
use crate::utils::response::common;

const GITLAB_LOGIN_STATE_KEY: &str = "login.gitlab";

#[derive(Clone, Serialize, Deserialize)]
struct GitlabLoginState {
    state: CsrfToken,
    attach: bool,
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Author>,
    mut session: WritableSession,
) -> Result<Either<(StatusCode, Html<String>), Redirect>, FrontendError> {
    if let Some(author) = maybe_author {
        let state = state.as_ref();
        let response = common::already_logged_in(state, author)?;
        return Ok(Either::E1(response));
    }

    let Some(gitlab_state) = state.frontend.auth.gitlab.as_ref() else {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "authentication using GitLab is not allowed on this instance",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    };

    let (url, state) = gitlab_state
        .client
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("read_api".to_string()))
        .url();

    let data = GitlabLoginState {
        state,
        attach: false,
    };
    session.insert(GITLAB_LOGIN_STATE_KEY, &data)?;

    return Ok(Either::E2(Redirect::to(url.as_str())));
}
