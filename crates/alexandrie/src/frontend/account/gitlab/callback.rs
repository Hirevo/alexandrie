use std::time::Duration;

use diesel::prelude::*;
use oauth2::reqwest::async_http_client;
use oauth2::{AccessToken, AuthorizationCode, TokenResponse};
use once_cell::sync::Lazy;
use regex::Regex;
use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};
use url::Url;

use crate::db::models::{Author, NewAuthor, NewSalt};
use crate::db::schema::{authors, salts};
use crate::frontend::account::gitlab::GITLAB_LOGIN_STATE_KEY;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

use super::GitlabLoginState;

static LINK_HEADER_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r#"<([^>]+)>; rel="next""#).unwrap());

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CallbackQueryData {
    code: String,
    state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitlabUser {
    id: u64,
    email: String,
    username: String,
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GitlabGroup {
    id: u64,
    name: String,
    path: String,
    full_name: String,
    full_path: String,
    parent_id: Option<u64>,
}

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    let data: GitlabLoginState = match req.session().get(GITLAB_LOGIN_STATE_KEY) {
        Some(data) => data,
        None => {
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::BadRequest,
                "no authentication is currently being performed",
            );
        }
    };
    req.session_mut().remove("login.gitlab");

    let current_author = match (data.attach, req.get_author()) {
        (true, Some(author)) => Some(author),
        (true, None) => {
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::BadRequest,
                "attaching to an account requires to be logged-in",
            );
        }
        (false, Some(_)) => {
            return Ok(utils::response::redirect("/"));
        }
        (false, None) => None,
    };

    let query: CallbackQueryData = req.query()?;

    let gitlab_config = &req.state().frontend.config.auth.gitlab;
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

    let allow_register = gitlab_config.allow_registration;

    if query.state.as_str() != data.state.secret() {
        return utils::response::error_html(
            req.state(),
            None,
            StatusCode::BadRequest,
            "CSRF token is different than expected",
        );
    }

    let code = AuthorizationCode::new(query.code);
    let token_response = gitlab_state
        .client
        .exchange_code(code)
        .request_async(async_http_client)
        .await?;

    let token = token_response.access_token();

    let client = reqwest::Client::builder()
        .user_agent("alexandrie/0.1.0")
        .build()?;

    let mut request_url = gitlab_config.origin.clone();
    request_url.set_path("/api/v4/user");

    let user_info: GitlabUser = client
        .get(request_url)
        .bearer_auth(token.secret().clone())
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    if let Some(allowed_groups) = gitlab_config.allowed_groups.as_ref() {
        let allowed =
            check_memberships(&gitlab_config.origin, allowed_groups, &client, &token).await?;

        if !allowed {
            //? The error message presented to users is intentionally vague about what check failed.
            // TODO(hirevo): maybe it is ok to say that it is an organization membership issue ?
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::NotFound,
                "GitLab user doesn't fulfill required conditions for access",
            );
        }
    }

    let state = req.state().clone();
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        let gitlab_id = user_info.id.to_string();

        //? If we are attaching to an existing account:
        if let Some(current_author) = current_author {
            //? Attach the GitLab account to this author.
            //?
            //? This will fail if the GitLab account is already claimed,
            //? because the `gitlab_id` column is marked `unique`.
            diesel::update(authors::table.find(current_author.id))
                .set(authors::gitlab_id.eq(gitlab_id.as_str()))
                .execute(conn)?;

            return Ok(utils::response::redirect("/"));
        }

        //? Is this GitLab account attached to an existing author ?
        let maybe_author: Option<Author> = authors::table
            .filter(authors::gitlab_id.eq(gitlab_id.as_str()))
            .first(conn)
            .optional()?;

        let author_id = if let Some(found_author) = maybe_author {
            // TODO (hirevo): (maybe) update user's details to the ones from Gitlab ?
            // TODO (hirevo): add mechanism for linking GitLab account to an already existing account.

            found_author.id
        } else if allow_register {
            //? Generate the user's authentication salt.
            let decoded_generated_salt = {
                let mut data = [0u8; 16];
                let rng = SystemRandom::new();
                rng.fill(&mut data).unwrap();
                hasher::digest(&hasher::SHA512, data.as_ref())
            };

            //? Insert the new author data.
            let new_author = NewAuthor {
                email: user_info.email.as_str(),
                name: user_info
                    .name
                    .as_deref()
                    .unwrap_or(user_info.username.as_str()),
                passwd: None,
                github_id: None,
                gitlab_id: Some(gitlab_id.as_str()),
            };
            diesel::insert_into(authors::table)
                .values(new_author)
                .execute(conn)?;

            //? Fetch the newly-inserted author back.
            let author_id = authors::table
                .select(authors::id)
                .filter(authors::gitlab_id.eq(gitlab_id.as_str()))
                .first::<i64>(conn)?;

            //? Store the author's newly-generated authentication salt.
            let encoded_generated_salt = hex::encode(decoded_generated_salt.as_ref());
            let new_salt = NewSalt {
                author_id,
                salt: encoded_generated_salt.as_str(),
            };
            diesel::insert_into(salts::table)
                .values(new_salt)
                .execute(conn)?;

            // TODO (hirevo): implement team membership verification (as an option).
            // TODO (hirevo): implement organization membership verification (as an option).

            author_id
        } else {
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::NotFound,
                "user registration is forbidden for this instance",
            );
        };

        //? Get the maximum duration of the session.
        let expiry = Duration::from_secs(86_400); // 1 day / 24 hours

        //? Set the user's session.
        req.session_mut().insert("author.id", author_id)?;
        req.session_mut().expire_in(expiry);

        return Ok(utils::response::redirect("/"));
    });

    transaction.await
}

async fn check_memberships(
    origin: &Url,
    allowed_groups: &[String],
    client: &reqwest::Client,
    token: &AccessToken,
) -> tide::Result<bool> {
    let mut request_url = origin.clone();
    request_url.set_path("/api/v4/groups");

    let mut next = Some(request_url);
    while let Some(url) = next {
        let response = client
            .get(url)
            .bearer_auth(token.secret().clone())
            .send()
            .await?
            .error_for_status()?;

        next = (|| {
            let value = response.headers().get("Link")?;
            let value = value.to_str().ok()?;
            let captures = LINK_HEADER_REGEX.captures(value)?;
            let capture = captures.get(1)?;
            let capture = Url::parse(capture.as_str()).ok()?;
            Some(capture)
        })();

        let groups: Vec<GitlabGroup> = response.json().await?;

        let allowed = groups
            .iter()
            .any(|it| allowed_groups.contains(&it.full_path));

        if allowed {
            return Ok(true);
        }
    }

    Ok(false)
}
