use std::sync::Arc;

use axum::extract::{Path, State};
use axum::{Json, TypedHeader};
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};

use crate::config::AppState;
use crate::db::models::{Author, NewCrateAuthor};
use crate::db::schema::*;
use crate::error::{AlexError, ApiError};
use crate::utils;
use crate::utils::auth::Authorization;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct OwnerListResponse {
    pub users: Vec<OwnerListEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct OwnerListEntry {
    pub id: i64,
    pub login: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct OwnerAddBody {
    /// Owners' emails to add.
    pub users: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct OwnerDeleteBody {
    /// Owners' emails to delete.
    pub users: Vec<String>,
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<OwnerListResponse>, ApiError> {
    let name = utils::canonical_name(name);

    let db = &state.db;
    let transaction = db.transaction(move |conn| {
        //? Does this crate exists?
        let exists = utils::checks::crate_exists(conn, name.as_str())?;
        if !exists {
            return Err(ApiError::msg(format!(
                "no crates named '{name}' could be found",
            )));
        }

        //? Get all authors of this crate.
        let authors = crate_authors::table
            .inner_join(authors::table)
            .inner_join(crates::table)
            .select(authors::all_columns)
            .filter(crates::canon_name.eq(name.as_str()))
            .load::<Author>(conn)?;

        let users = authors
            .into_iter()
            .map(|author| {
                let Author {
                    id,
                    email: login,
                    name,
                    ..
                } = author;
                OwnerListEntry { id, login, name }
            })
            .collect();

        Ok(Json(OwnerListResponse { users }))
    });

    transaction.await.map_err(ApiError::from)
}

pub(crate) async fn put(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization>,
    Json(body): Json<OwnerAddBody>,
) -> Result<Json<json::Value>, ApiError> {
    let name = utils::canonical_name(name);

    let OwnerAddBody { users: new_authors } = body;

    let db = &state.db;
    let transaction = db.transaction(move |conn| {
        let header = authorization.token().to_string();
        let author = utils::checks::get_author(conn, header).ok_or(AlexError::InvalidToken)?;

        //? Get this crate's ID.
        let maybe_crate_id = crates::table
            .select(crates::id)
            .filter(crates::canon_name.eq(name.as_str()))
            .first::<i64>(conn)
            .optional()?;
        let Some(crate_id) = maybe_crate_id else {
            return Err(ApiError::msg(
                format!("no crates named '{name}' could be found"),
            ));
        };

        //? Get all authors of this crate.
        let crate_authors = crate_authors::table
            .inner_join(authors::table)
            .inner_join(crates::table)
            .select(authors::id)
            .filter(crates::id.eq(crate_id))
            .load::<i64>(conn)?;

        //? Check if user is one of these authors.
        if !crate_authors.contains(&author.id) {
            return Err(ApiError::msg("you are not an author of this crate"));
        }

        //? Get all registered authors which:
        //?   - are not authors of this crate.
        //?   - are one of the requested new authors.
        let new_authors = authors::table
            .select((authors::id, authors::name))
            .filter(authors::id.ne_all(crate_authors.as_slice()))
            .filter(authors::email.eq_any(new_authors.as_slice()))
            .load::<(i64, String)>(conn)?;

        let (new_authors_ids, new_authors_names) = {
            let len = new_authors.len();
            let (mut fsts, mut snds) = (Vec::with_capacity(len), Vec::with_capacity(len));
            for (fst, snd) in new_authors.into_iter() {
                fsts.push(fst);
                snds.push(snd);
            }
            (fsts, snds)
        };

        //? Prepare requests structures.
        let new_authors = new_authors_ids
            .into_iter()
            .map(|author_id| NewCrateAuthor {
                crate_id,
                author_id,
            })
            .collect::<Vec<_>>();

        //? Insert the new authors.
        diesel::insert_into(crate_authors::table)
            .values(new_authors)
            .execute(conn)?;

        let authors_list = match new_authors_names.as_slice() {
            [] => String::new(),
            [author] => author.clone(),
            [fst, snd] => format!("{fst}, and {snd}"),
            [fsts @ .., last] => {
                let fsts = fsts.join(", ");
                format!("{fsts}, and {last}")
            }
        };

        Ok(Json(json!({
            "ok": true,
            "msg": format!("{authors_list} has been added as authors of {name}"),
        })))
    });

    transaction.await
}

pub(crate) async fn delete(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
    TypedHeader(authorization): TypedHeader<Authorization>,
    Json(body): Json<OwnerDeleteBody>,
) -> Result<Json<json::Value>, ApiError> {
    let name = utils::canonical_name(name);

    let OwnerDeleteBody { users: old_authors } = body;

    let db = &state.db;
    let transaction = db.transaction(move |conn| {
        let header = authorization.token().to_string();
        let author = utils::checks::get_author(conn, header).ok_or(AlexError::InvalidToken)?;

        //? Get this crate's ID.
        let maybe_crate_id = crates::table
            .select(crates::id)
            .filter(crates::canon_name.eq(name.as_str()))
            .first::<i64>(conn)
            .optional()?;
        let Some(crate_id) = maybe_crate_id else {
            return Err(ApiError::msg(
                format!("no crates named '{name}' could be found"),
            ));
        };

        //? Get all authors of this crate.
        let crate_authors = crate_authors::table
            .inner_join(authors::table)
            .inner_join(crates::table)
            .select(authors::id)
            .filter(crates::id.eq(crate_id))
            .load::<i64>(conn)?;

        //? Check if user is one of these authors.
        if !crate_authors.contains(&author.id) {
            return Err(ApiError::msg("you are not an author of this crate"));
        }

        //? Get all registered authors which:
        //?   - are authors of this crate.
        //?   - are one of the requested new authors.
        let old_authors = authors::table
            .select((authors::id, authors::name))
            .filter(authors::id.eq_any(crate_authors.as_slice()))
            .filter(authors::email.eq_any(old_authors.as_slice()))
            .load::<(i64, String)>(conn)?;

        //? Check if there will remain at least one author for this crate.
        if crate_authors.len() == 1 && old_authors.len() == 1 {
            return Err(ApiError::msg("cannot leave the crate without any authors"));
        }

        //? Split IDs and names into separate vectors.
        let (old_authors_ids, old_authors_names) = {
            let len = old_authors.len();
            let (mut fsts, mut snds) = (Vec::with_capacity(len), Vec::with_capacity(len));
            for (fst, snd) in old_authors.into_iter() {
                fsts.push(fst);
                snds.push(snd);
            }
            (fsts, snds)
        };

        //? Delete from authors.
        diesel::delete(
            crate_authors::table
                .filter(crate_authors::crate_id.eq(crate_id))
                .filter(crate_authors::author_id.eq_any(old_authors_ids.as_slice())),
        )
        .execute(conn)?;

        let authors_list = match old_authors_names.as_slice() {
            [] => String::new(),
            [author] => author.clone(),
            [fst, snd] => format!("{fst}, and {snd}"),
            [fsts @ .., last] => {
                let fsts = fsts.join(", ");
                format!("{fsts}, and {last}")
            }
        };

        Ok(Json(json!({
            "ok": true,
            "msg": format!("{authors_list} has been removed from authors of {name}"),
        })))
    });

    transaction.await.map_err(ApiError::from)
}
