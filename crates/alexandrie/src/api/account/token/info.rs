use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::AppState;
use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::error::ApiError;
use crate::utils::auth::api::Auth;

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
pub async fn get(
    State(state): State<Arc<AppState>>,
    Auth(author): Auth,
    Path(name): Path<String>,
) -> Result<Json<ResponseBody>, ApiError> {
    let db = &state.db;

    //? Fetch the token from the database.
    let maybe_token = db
        .run(move |conn| {
            author_tokens::table
                .filter(author_tokens::name.eq(name.as_str()))
                .filter(author_tokens::author_id.eq(author.id))
                .first::<AuthorToken>(conn)
                .optional()
        })
        .await?;

    //? Was a token found ?
    let Some(token) = maybe_token else {
        return Err(ApiError::msg("no token was found for the supplied name"));
    };

    Ok(Json(ResponseBody {
        name: token.name,
        expires_at: None,
    }))
}

/// Route to get information about a registry token.
pub async fn post(
    State(state): State<Arc<AppState>>,
    Auth(author): Auth,
    Json(body): Json<RequestBody>,
) -> Result<Json<ResponseBody>, ApiError> {
    let db = &state.db;

    //? Fetch the token from the database.
    let maybe_token = db
        .run(move |conn| {
            author_tokens::table
                .filter(author_tokens::token.eq(body.token.as_str()))
                .filter(author_tokens::author_id.eq(author.id))
                .first::<AuthorToken>(conn)
                .optional()
        })
        .await?;

    //? Was a token found ?
    let Some(token) = maybe_token else {
        return Err(ApiError::msg("unauthorized access to this token"));
    };

    Ok(Json(ResponseBody {
        name: token.name,
        expires_at: None,
    }))
}
