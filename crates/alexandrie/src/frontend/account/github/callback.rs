use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use tower_sessions::Session;
use diesel::prelude::*;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse};
use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};

use crate::config::frontend::auth::github::GithubAuthOrganizationConfig;
use crate::config::AppState;
use crate::db::models::{Author, NewAuthor, NewSalt};
use crate::db::schema::{authors, salts};
use crate::error::FrontendError;
use crate::frontend::account::github::GITHUB_LOGIN_STATE_KEY;
use crate::utils;
use crate::utils::auth::frontend::Auth;

use super::GithubLoginState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CallbackQueryData {
    code: String,
    state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubUser {
    id: u64,
    login: String,
    name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubUserEmail {
    email: String,
    verified: bool,
    primary: bool,
    visibility: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubUserOrg {
    id: u64,
    login: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubUserTeam {
    id: u64,
    organization: GithubUserTeamOrg,
    // "parent": null,
    slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct GithubUserTeamOrg {
    id: u64,
    login: String,
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Query(query): Query<CallbackQueryData>,
    maybe_author: Option<Auth>,
    session: Session,
) -> Result<Either<(StatusCode, Html<String>), Redirect>, FrontendError> {
    let Some(data): Option<GithubLoginState> = session.remove(GITHUB_LOGIN_STATE_KEY)? else {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "no authentication is currently being performed",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    };

    let current_author = match (data.attach, maybe_author) {
        (true, Some(author)) => Some(author),
        (true, None) => {
            let rendered = utils::response::error_html(
                state.as_ref(),
                None,
                "attaching to an account requires to be logged-in",
            )?;
            return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
        }
        (false, Some(_)) => {
            return Ok(Either::E2(Redirect::to("/")));
        }
        (false, None) => None,
    };

    let github_config = &state.frontend.config.auth.github;
    let Some(github_state) = state.frontend.auth.github.as_ref() else {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "authentication using GitHub is not allowed on this instance",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    };

    let allow_register = github_config.allow_registration;

    if query.state.as_str() != data.state.secret() {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "CSRF token is different than expected",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    }

    let code = AuthorizationCode::new(query.code);
    let token_response = github_state
        .client
        .exchange_code(code)
        .request_async(async_http_client)
        .await?;

    let token = token_response.access_token();
    let token = format!("token {}", token.secret());

    //? The GitHub REST API v3 requires a user-agent string with the name of the application.
    let client = reqwest::Client::builder()
        .user_agent("alexandrie/0.1.0")
        .build()?;

    //? Get GitHub user information.
    let user_info: GithubUser = client
        .get("https://api.github.com/user")
        .header("accept", "application/vnd.github.v3+json")
        .header("authorization", &token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    //? Get the list of their email addresses (to find their primary email address).
    let emails: Vec<GithubUserEmail> = client
        .get("https://api.github.com/user/emails")
        .header("accept", "application/vnd.github.v3+json")
        .header("authorization", &token)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    //? Find the primary email address.
    let maybe_primary_email = emails.into_iter().find(|email| email.primary == true);
    let Some(primary_email) = maybe_primary_email else {
        let rendered =
            utils::response::error_html(state.as_ref(), None, "could not find primary email")?;
        return Ok(Either::E1((StatusCode::NOT_FOUND, Html(rendered))));
    };

    if let Some(allowed_organizations) = github_config.allowed_organizations.as_ref() {
        //? Get list of user organizations that are also specified in the configuration.
        let found_orgs = fetch_relevent_user_orgs(allowed_organizations, &client, &token).await?;

        //? If the configuration doesn't require specific team memberships for any of these orgs, the user is then already allowed.
        let already_allowed = found_orgs.iter().any(|it| it.allowed_teams.is_none());

        if found_orgs.is_empty()
            || (!already_allowed
                && !check_team_memberships(allowed_organizations, &client, &token).await?)
        {
            //? The error message presented to users is intentionally vague about what check failed.
            // TODO(hirevo): maybe it is ok to say that it is an organization membership issue ?
            let rendered = utils::response::error_html(
                state.as_ref(),
                None,
                "GitHub user doesn't fulfill required conditions for access",
            )?;
            return Ok(Either::E1((StatusCode::NOT_FOUND, Html(rendered))));
        }
    }

    let db = &state.db;
    let state = Arc::clone(&state);

    let transaction = db.transaction(move |conn| {
        let github_id = user_info.id.to_string();

        //? If we are attaching to an existing account:
        if let Some(current_author) = current_author {
            //? Attach the GitHub account to this author.
            //?
            //? This will fail if the GitHub account is already claimed,
            //? because the `github_id` column is marked `unique`.
            diesel::update(authors::table.find(current_author.id))
                .set(authors::github_id.eq(github_id.as_str()))
                .execute(conn)?;

            return Ok(Either::E2(Redirect::to("/")));
        }

        //? Is this GitHub account attached to an existing author ?
        let maybe_author: Option<Author> = authors::table
            .filter(authors::github_id.eq(github_id.as_str()))
            .first(conn)
            .optional()?;

        let author_id = if let Some(found_author) = maybe_author {
            // TODO (hirevo): (maybe) update user's details to the ones from GitHub ?
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
                email: primary_email.email.as_str(),
                name: user_info
                    .name
                    .as_deref()
                    .unwrap_or(user_info.login.as_str()),
                passwd: None,
                github_id: Some(github_id.as_str()),
                gitlab_id: None,
            };
            diesel::insert_into(authors::table)
                .values(new_author)
                .execute(conn)?;

            //? Fetch the newly-inserted author back.
            let author_id = authors::table
                .select(authors::id)
                .filter(authors::github_id.eq(github_id.as_str()))
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

            author_id
        } else {
            let rendered = utils::response::error_html(
                state.as_ref(),
                None,
                "user registration is forbidden for this instance",
            )?;
            return Ok(Either::E1((StatusCode::NOT_FOUND, Html(rendered))));
        };

        //? Get the maximum duration of the session.
        let expiry = time::Duration::seconds(86_400); // 1 day / 24 hours

        //? Set the user's session.
        session.insert("author.id", author_id)?;
        session.expire_in(expiry);

        return Ok(Either::E2(Redirect::to("/")));
    });

    transaction.await
}

async fn fetch_relevent_user_orgs<'a>(
    allowed_organizations: &'a [GithubAuthOrganizationConfig],
    client: &reqwest::Client,
    token: &str,
) -> Result<Vec<&'a GithubAuthOrganizationConfig>, FrontendError> {
    let mut found_orgs = Vec::new();

    for page_number in 1.. {
        //? Get the list of the user's organizations.
        let orgs: Vec<GithubUserOrg> = client
            .get(format!(
                "https://api.github.com/user/orgs?page={}",
                page_number
            ))
            .header("accept", "application/vnd.github.v3+json")
            .header("authorization", token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        //? No more organizations to be listed.
        if orgs.is_empty() {
            break;
        }

        //? Checks that the GitHub user is within one of the allowed organizations.
        let iter = orgs
            .iter()
            .filter_map(|org| allowed_organizations.iter().find(|it| it.name == org.login));
        found_orgs.extend(iter);
    }

    Ok(found_orgs)
}

async fn check_team_memberships(
    allowed_organizations: &[GithubAuthOrganizationConfig],
    client: &reqwest::Client,
    token: &str,
) -> Result<bool, FrontendError> {
    for page_number in 1.. {
        //? Get the list of the user's teams (which includes organizations).
        let teams: Vec<GithubUserTeam> = client
            .get(format!(
                "https://api.github.com/user/teams?page={}",
                page_number
            ))
            .header("accept", "application/vnd.github.v3+json")
            .header("authorization", token)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;

        //? No more teams to be listed.
        if teams.is_empty() {
            break;
        }

        //? Checks that the GitHub user is within one of the allowed teams within one of the allowed organizations.
        let allowed = teams.iter().any(|team| {
            allowed_organizations.iter().any(|it| {
                it.name == team.organization.login
                    && (it.allowed_teams.as_ref()).map_or(true, |it| it.contains(&team.slug))
            })
        });

        if allowed {
            return Ok(true);
        }
    }

    Ok(false)
}
