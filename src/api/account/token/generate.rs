use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::db::models::{AuthorToken, NewAuthorToken};
use crate::db::schema::*;
use crate::utils;
use crate::State;

/// Request body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBody {
    /// The registry token to revoke.
    pub name: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// A registry token for that account.
    pub token: String,
}

/// Route to revoke a registry token.
pub async fn put(mut req: Request<State>) -> tide::Result {
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
                "please log in first to generate tokens",
            ));
        }
    };

    //? Parse request body.
    let body: RequestBody = req.body_json().await?;

    let transaction = repo.transaction(move |conn| {
        //? Does a token with that name already exist for that author ?
        let token = author_tokens::table
            .filter(author_tokens::name.eq(body.name.as_str()))
            .filter(author_tokens::author_id.eq(author.id))
            .first::<AuthorToken>(conn)
            .optional()?;

        //? Was a token found ?
        if token.is_some() {
            return Ok(utils::response::error(
                StatusCode::BadRequest,
                "a token of that same name already exist for your account",
            ));
        }

        //? Generate new registry token.
        let account_token = utils::auth::generate_token();
        let (token, _) = account_token.split_at(25);

        //? Store the new registry token in the database.
        let new_author_token = NewAuthorToken {
            token,
            name: body.name.as_str(),
            author_id: author.id,
        };

        //? Insert the token into the database.
        let _ = diesel::insert_into(author_tokens::table)
            .values(new_author_token)
            .execute(conn)?;

        let response = ResponseBody {
            token: String::from(token),
        };
        Ok(utils::response::json(&response))
    });

    transaction.await
}
