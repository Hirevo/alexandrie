use oauth2::{CsrfToken, Scope};
use tide::{Request, StatusCode};

use crate::frontend::account::gitlab::{GitlabLoginState, GITLAB_LOGIN_STATE_KEY};
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    if !req.is_authenticated() {
        return Ok(utils::response::redirect("/account/manage"));
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
        attach: true,
    };
    req.session_mut().insert(GITLAB_LOGIN_STATE_KEY, &data)?;

    return Ok(utils::response::redirect(url.as_str()));
}
