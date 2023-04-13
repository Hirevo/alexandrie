use oauth2::{CsrfToken, Scope};
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

/// Endpoint to attach to an existing Alexandrie account.
pub mod attach;
/// Callback endpoint for the "gitlab" authentication strategy.
pub mod callback;
/// Endpoint to detach from an existing Alexandrie account.
pub mod detach;

use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::response::common;
use crate::State;

const GITLAB_LOGIN_STATE_KEY: &str = "login.gitlab";

#[derive(Clone, Serialize, Deserialize)]
struct GitlabLoginState {
    state: CsrfToken,
    attach: bool,
}

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    if let Some(author) = req.get_author() {
        let state = req.state().as_ref();
        return common::already_logged_in(state, author);
    }

    let gitlab_state = match req.state().frontend.auth.gitlab.as_ref() {
        Some(state) => state,
        None => {
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::BadRequest,
                "authentication using GitLab is not allowed on this instance",
            );
        }
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
    req.session_mut().insert(GITLAB_LOGIN_STATE_KEY, &data)?;

    return Ok(utils::response::redirect(url.as_str()));
}
