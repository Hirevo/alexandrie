use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use diesel::prelude::*;
use json::json;
use semver::Version;

use alexandrie_index::Indexer;

use crate::config::AppState;
use crate::db::schema::crates;
use crate::error::ApiError;
use crate::utils;
use crate::utils::auth::api::Auth;

pub(crate) async fn put(
    State(state): State<Arc<AppState>>,
    Auth(author): Auth,
    Path((name, version)): Path<(String, Version)>,
) -> Result<Json<json::Value>, ApiError> {
    let name = utils::canonical_name(name);

    let db = &state.db;
    let state = Arc::clone(&state);
    let transaction = db.transaction(move |conn| {
        //? Does this crate exists?
        let exists = utils::checks::crate_exists(conn, name.as_str())?;
        if !exists {
            return Err(ApiError::msg(format!(
                "no crates named '{name}' could be found"
            )));
        }

        //? Is the user an author of this crate?
        let is_author = utils::checks::is_crate_author(conn, name.as_str(), author.id)?;
        if !is_author {
            return Err(ApiError::msg("you are not an author of this crate"));
        }

        //? Get the non-canonical crate name from the canonical one.
        let name = crates::table
            .select(crates::name)
            .filter(crates::canon_name.eq(name.as_str()))
            .first::<String>(conn)?;

        state.index.unyank_record(name.as_str(), version.clone())?;

        let msg = format!("Unyanking crate `{name}#{version}`");
        state.index.commit_and_push(msg.as_str())?;

        Ok(Json(json!({
            "ok": true
        })))
    });

    transaction.await.map_err(ApiError::from)
}
