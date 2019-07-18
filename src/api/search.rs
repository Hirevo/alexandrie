use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use rocket::State;
use rocket_contrib::json::Json;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::db::DbConn;
use crate::error::Error;
use crate::index::Indexer;
use crate::state::AppState;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResponse {
    pub crates: Vec<SearchResult>,
    pub meta: SearchMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub max_version: Version,
    pub description: Option<String>,
    pub downloads: u64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub documentation: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMeta {
    pub total: u64,
}

/// Route to search through crates (used by `cargo search`).
#[get("/crates?<q>&<per_page>&<page>")]
pub(crate) fn route(
    state: State<Arc<Mutex<AppState>>>,
    conn: DbConn,
    q: String,
    per_page: Option<u32>,
    page: Option<u32>,
) -> Result<Json<SearchResponse>, Error> {
    let state = state.lock().unwrap();

    //? Fetch the latest index changes.
    state.index().refresh()?;

    //? Build the search pattern.
    let name_pattern = format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%"));
    let req = crates::table
        .filter(crates::name.like(name_pattern.as_str()))
        .into_boxed();

    //? Limit the result count depending on parameters.
    let req = match (per_page, page) {
        (Some(per_page), Some(page)) => req
            .limit(i64::from(per_page))
            .offset(i64::from(page * per_page)),
        (Some(per_page), None) => req.limit(i64::from(per_page)),
        _ => req,
    };
    let results = req.load::<CrateRegistration>(&conn.0)?;

    //? Fetch the total result count.
    let total = crates::table
        .select(diesel::dsl::count(crates::name))
        .filter(crates::name.like(name_pattern.as_str()))
        .first::<i64>(&conn.0)?;

    let crates = results
        .into_iter()
        .map(|krate| {
            let latest = state.index().latest_crate(krate.name.as_str())?;
            Ok(SearchResult {
                name: krate.name,
                max_version: latest.vers,
                description: krate.description,
                downloads: krate.downloads,
                created_at: krate.created_at,
                updated_at: krate.updated_at,
                documentation: krate.documentation,
                repository: krate.repository,
            })
        })
        .collect::<Result<Vec<SearchResult>, Error>>()?;

    Ok(Json(SearchResponse {
        crates,
        meta: SearchMeta {
            total: total as u64,
        },
    }))
}
