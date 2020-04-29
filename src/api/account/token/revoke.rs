use diesel::prelude::*;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::utils;
use crate::{Error, State};

/// Request body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBody {
    /// The registry token to get information about.
    pub token: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// Has the token been revoked ?
    pub revoked: bool,
}

/// Route to revoke a registry token.
pub async fn delete(mut req: Request<State>) -> Result<Response, Error> {
    let state = req.state().clone();
    let repo = &state.repo;

    //? Is the author logged in ?
    let headers = req.headers().clone();
    let author = repo
        .run(move |conn| utils::checks::get_author(conn, &headers))
        .await;
    let author = match author {
        Some(author) => author,
        None => {
            return Ok(utils::response::error(
                StatusCode::UNAUTHORIZED,
                "please log in first to revoke tokens",
            ));
        }
    };

    //? Parse request body.
    let body: RequestBody = req.body_json().await?;

    let transaction = repo.transaction(move |conn| {
        //? Fetch the token from the database.
        let token = author_tokens::table
            .filter(author_tokens::token.eq(body.token.as_str()))
            .first::<AuthorToken>(conn)
            .optional()?;

        //? Was a token found ?
        let token = match token {
            Some(token) => token,
            None => {
                return Ok(utils::response::error(
                    StatusCode::FORBIDDEN,
                    "unauthorized access to this token",
                ))
            }
        };

        //? Is the token from that same author ?
        if token.author_id != author.id {
            return Ok(utils::response::error(
                StatusCode::FORBIDDEN,
                "unauthorized access to this token",
            ));
        }

        //? Revoke that token.
        diesel::delete(author_tokens::table.filter(author_tokens::id.eq(token.id)))
            .execute(conn)?;

        let response = ResponseBody { revoked: true };
        Ok(utils::response::json(&response))
    });

    transaction.await
}
