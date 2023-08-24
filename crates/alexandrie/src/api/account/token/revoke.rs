use std::sync::Arc;

use anyhow::Context;
use axum::extract::State;
use axum::{Json, TypedHeader};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::AppState;
use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::error::ApiError;
use crate::utils;
use crate::utils::auth::Authorization;

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
pub async fn delete(
    State(state): State<Arc<AppState>>,
    TypedHeader(authorization): TypedHeader<Authorization>,
    Json(body): Json<RequestBody>,
) -> Result<Json<ResponseBody>, ApiError> {
    let db = &state.db;

    //? Is the author logged in ?
    let author = {
        let token = authorization.token().to_string();
        db.run(move |conn| utils::checks::get_author(conn, token))
            .await
            .context("invalid authorization token")?
    };

    let transaction = db.transaction(move |conn| {
        //? Fetch the token from the database.
        let maybe_token = author_tokens::table
            .filter(author_tokens::token.eq(body.token.as_str()))
            .first::<AuthorToken>(conn)
            .optional()?;

        //? Was a token found ?
        let Some(token) = maybe_token else {
            return Err(ApiError::msg("unauthorized access to this token"));
        };

        //? Is the token from that same author ?
        if token.author_id != author.id {
            return Err(ApiError::msg("unauthorized access to this token"));
        }

        //? Revoke that token.
        diesel::delete(author_tokens::table.filter(author_tokens::id.eq(token.id)))
            .execute(conn)?;

        Ok(Json(ResponseBody { revoked: true }))
    });

    transaction.await.map_err(ApiError::from)
}
