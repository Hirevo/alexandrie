use std::num::NonZeroU32;

use diesel::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use tide::querystring::ContextExt;
use tide::{Context, Response};

use crate::config::State;
use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::error::Error;
use crate::index::Indexer;

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchParams {
    q: String,
    per_page: Option<NonZeroU32>,
    page: Option<NonZeroU32>,
}

/// Route to search through crates (used by `cargo search`).
pub(crate) async fn route(ctx: Context<State>) -> Result<Response, Error> {
    let params = ctx.url_query::<SearchParams>().unwrap();
    let state = ctx.state();
    let repo = &state.repo;

    //? Fetch the latest index changes.
    state.index.refresh()?;

    //? Build the search pattern.
    let name_pattern = format!("%{}%", params.q.replace('\\', "\\\\").replace('%', "\\%"));

    //? Limit the result count depending on parameters.
    let results = match (params.per_page, params.page) {
        (Some(per_page), Some(page)) => {
            repo.run(|conn| {
                crates::table
                    .filter(crates::name.like(name_pattern.as_str()))
                    .limit(i64::from(per_page.get()))
                    .offset(i64::from((page.get() - 1) * per_page.get()))
                    .load::<CrateRegistration>(&conn)
            })
            .await?
        }
        (Some(per_page), None) => {
            repo.run(|conn| {
                crates::table
                    .filter(crates::name.like(name_pattern.as_str()))
                    .limit(i64::from(per_page.get()))
                    .load::<CrateRegistration>(&conn)
            })
            .await?
        }
        _ => {
            repo.run(|conn| {
                crates::table
                    .filter(crates::name.like(name_pattern.as_str()))
                    .load::<CrateRegistration>(&conn)
            })
            .await?
        }
    };

    //? Fetch the total result count.
    let total = repo
        .run(|conn| {
            crates::table
                .select(diesel::dsl::count(crates::name))
                .filter(crates::name.like(name_pattern.as_str()))
                .first::<i64>(&conn)
        })
        .await?;

    let crates = results
        .into_iter()
        .map(|krate| {
            let latest = state.index.latest_crate(krate.name.as_str())?;
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

    Ok(tide::response::json(SearchResponse {
        crates,
        meta: SearchMeta {
            total: total as u64,
        },
    }))
}
