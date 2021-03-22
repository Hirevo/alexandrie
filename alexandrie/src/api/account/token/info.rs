use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::utils;
use crate::State;

/// Request body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBody {
    /// The registry token to get information about.
    pub token: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// The token name.
    pub name: String,
    /// The expiration date for this token.
    pub expires_at: Option<String>,
}

/// Route to get information about a registry token.
pub async fn get(req: Request<State>) -> tide::Result {
    let name = req.param("name")?.to_string();

    let state = req.state().clone();
    let repo = &state.repo;

    //? Is the author logged in ?
    let author = if let Some(headers) = req.header(utils::auth::AUTHORIZATION_HEADER) {
        let header = headers.last().to_string();
        repo.run(move |conn| utils::checks::get_author(conn, header))
            .await
    } else {
        None
    };
    let author = match author {
        Some(author) => author,
        None => {
            return Ok(utils::response::error(
                StatusCode::Unauthorized,
                "please log in first to access token information",
            ));
        }
    };

    //? Fetch the token from the database.
    let token = repo
        .run(move |conn| {
            author_tokens::table
                .filter(author_tokens::name.eq(name.as_str()))
                .filter(author_tokens::author_id.eq(author.id))
                .first::<AuthorToken>(conn)
                .optional()
        })
        .await?;

    //? Was a token found ?
    let token = match token {
        Some(token) => token,
        None => {
            return Ok(utils::response::error(
                StatusCode::NotFound,
                "no token was found for the supplied name",
            ))
        }
    };

    let expires_at = None;
    let response = ResponseBody {
        name: token.name,
        expires_at,
    };
    Ok(utils::response::json(&response))
}

/// Route to get information about a registry token.
pub async fn post(mut req: Request<State>) -> tide::Result {
    let state = req.state().clone();
    let repo = &state.repo;

    //? Is the author logged in ?
    let author = if let Some(headers) = req.header(utils::auth::AUTHORIZATION_HEADER) {
        let header = headers.last().to_string();
        repo.run(move |conn| utils::checks::get_author(conn, header))
            .await
    } else {
        None
    };
    let author = match author {
        Some(author) => author,
        None => {
            return Ok(utils::response::error(
                StatusCode::Unauthorized,
                "please log in first to access token information",
            ));
        }
    };

    //? Parse request body.
    let body: RequestBody = req.body_json().await?;

    //? Fetch the token from the database.
    let token = repo
        .run(move |conn| {
            author_tokens::table
                .filter(author_tokens::token.eq(body.token.as_str()))
                .filter(author_tokens::author_id.eq(author.id))
                .first::<AuthorToken>(conn)
                .optional()
        })
        .await?;

    //? Was a token found ?
    let token = match token {
        Some(token) => token,
        None => {
            return Ok(utils::response::error(
                StatusCode::Forbidden,
                "unauthorized access to this token",
            ))
        }
    };

    let expires_at = None;
    let response = ResponseBody {
        name: token.name,
        expires_at,
    };
    Ok(utils::response::json(&response))
}
