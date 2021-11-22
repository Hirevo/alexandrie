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
    /// The registry token to revoke.
    pub token: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// Has the token been revoked ?
    pub revoked: bool,
}

/// Route to revoke a registry token.
pub async fn delete(mut req: Request<State>) -> tide::Result {
    let state = req.state().clone();
    let db = &state.db;

    //? Is the author logged in ?
    let author = if let Some(headers) = req.header(utils::auth::AUTHORIZATION_HEADER) {
        let header = headers.last().to_string();
        db.run(move |conn| utils::checks::get_author(conn, header))
            .await
    } else {
        None
    };
    let author = match author {
        Some(author) => author,
        None => {
            return Ok(utils::response::error(
                StatusCode::Unauthorized,
                "please log in first to revoke tokens",
            ));
        }
    };

    //? Parse request body.
    let body: RequestBody = req.body_json().await?;

    let transaction = db.transaction(move |conn| {
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
                    StatusCode::Forbidden,
                    "unauthorized access to this token",
                ))
            }
        };

        //? Is the token from that same author ?
        if token.author_id != author.id {
            return Ok(utils::response::error(
                StatusCode::Forbidden,
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
