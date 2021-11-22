use std::time::Duration;

use diesel::prelude::*;
use oauth2::reqwest::async_http_client;
use oauth2::{AuthorizationCode, TokenResponse};
use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::config::frontend::auth::github::GithubAuthOrganizationConfig;
use crate::db::models::{Author, NewAuthor, NewSalt};
use crate::db::schema::{authors, salts};
use crate::frontend::account::github::GITHUB_LOGIN_STATE_KEY;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

use super::GithubLoginState;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CallbackQueryData {
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

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    let data: GithubLoginState = match req.session().get(GITHUB_LOGIN_STATE_KEY) {
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
    req.session_mut().remove("login.github");

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

    let allow_register = github_config.allow_registration;

    if query.state.as_str() != data.state.secret() {
        return utils::response::error_html(
            req.state(),
            None,
            StatusCode::BadRequest,
            "CSRF token is different than expected",
        );
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
    let primary_email = match maybe_primary_email {
        Some(email) => email,
        None => {
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::NotFound,
                "could not find primary email",
            );
        }
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
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::NotFound,
                "GitHub user doesn't fulfill required conditions for access",
            );
        }
    }

    let state = req.state().clone();
    let db = &state.db;

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

            return Ok(utils::response::redirect("/"));
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

async fn fetch_relevent_user_orgs<'a>(
    allowed_organizations: &'a [GithubAuthOrganizationConfig],
    client: &reqwest::Client,
    token: &str,
) -> tide::Result<Vec<&'a GithubAuthOrganizationConfig>> {
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
) -> tide::Result<bool> {
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
