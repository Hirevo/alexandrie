use oauth2::{CsrfToken, Scope};
use tide::{Request, StatusCode};

use crate::frontend::account::github::{GithubLoginState, GITHUB_LOGIN_STATE_KEY};
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    if !req.is_authenticated() {
        return Ok(utils::response::redirect("/account/manage"));
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

    let (url, state) = builder.url();

    let data = GithubLoginState {
        state,
        attach: true,
    };
    req.session_mut().insert(GITHUB_LOGIN_STATE_KEY, &data)?;

    return Ok(utils::response::redirect(url.as_str()));
}
