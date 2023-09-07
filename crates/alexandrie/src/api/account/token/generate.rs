use std::sync::Arc;

use axum::extract::State;
use axum::Json;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::AppState;
use crate::db::models::{AuthorToken, NewAuthorToken};
use crate::db::schema::*;
use crate::error::ApiError;
use crate::utils;
use crate::utils::auth::api::Auth;

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
pub async fn put(
    State(state): State<Arc<AppState>>,
    Auth(author): Auth,
    Json(body): Json<RequestBody>,
) -> Result<Json<ResponseBody>, ApiError> {
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        //? Does a token with that name already exist for that author ?
        let token = author_tokens::table
            .filter(author_tokens::name.eq(body.name.as_str()))
            .filter(author_tokens::author_id.eq(author.id))
            .first::<AuthorToken>(conn)
            .optional()?;

        //? Was a token found ?
        if token.is_some() {
            return Err(ApiError::msg(
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

        Ok(Json(ResponseBody {
            token: String::from(token),
        }))
    });

    transaction.await.map_err(ApiError::from)
}
