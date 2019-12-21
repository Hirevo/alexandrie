use std::num::NonZeroU32;

use diesel::dsl as sql;
use diesel::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::{AlexError, Error};
use crate::index::Indexer;
use crate::utils;
use crate::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SearchResponse {
    pub crates: Vec<SearchResult>,
    pub meta: SearchMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SearchResult {
    pub name: String,
    pub max_version: Version,
    pub description: Option<String>,
    pub downloads: i64,
    pub created_at: chrono::NaiveDateTime,
    pub updated_at: chrono::NaiveDateTime,
    pub documentation: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SearchMeta {
    pub total: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SearchParams {
    pub q: String,
    pub per_page: Option<NonZeroU32>,
    pub page: Option<NonZeroU32>,
}

/// Route to search through crates (used by `cargo search`).
pub(crate) async fn get(req: Request<State>) -> Result<Response, Error> {
    let params = req
        .query::<SearchParams>()
        .map_err(|_| AlexError::MissingQueryParams {
            missing_params: &["q"],
        })?;
    let state = req.state().clone();
    let repo = &state.repo;

    //? Fetch the latest index changes.
    // state.index.refresh()?;

    //? Build the search pattern.
    let name_pattern = format!("%{0}%", params.q.replace('\\', "\\\\").replace('%', "\\%"));

    let transaction = repo.transaction(move |conn| {
        let state = req.state();

        //? Limit the result count depending on parameters.
        let results = match (params.per_page, params.page) {
            (Some(per_page), Some(page)) => {
                //? Get search results for the given page number and entries per page.
                crates::table
                    .filter(crates::name.like(name_pattern.as_str()))
                    .limit(i64::from(per_page.get()))
                    .offset(i64::from((page.get() - 1) * per_page.get()))
                    .load::<CrateRegistration>(conn)?
            }
            (Some(per_page), None) => {
                //? Get the first page of search results with the given entries per page.
                crates::table
                    .filter(crates::name.like(name_pattern.as_str()))
                    .limit(i64::from(per_page.get()))
                    .load::<CrateRegistration>(conn)?
            }
            _ => {
                //? Get ALL the crates (might be too much, tbh).
                crates::table
                    .filter(crates::name.like(name_pattern.as_str()))
                    .load::<CrateRegistration>(conn)?
            }
        };

        //? Fetch the total result count.
        let total = crates::table
            .select(sql::count(crates::name))
            .filter(crates::name.like(name_pattern.as_str()))
            .first::<i64>(conn)?;

        let crates = results
            .into_iter()
            .map(|krate| {
                let latest = state.index.latest_record(krate.name.as_str())?;
                let created_at = chrono::NaiveDateTime::parse_from_str(
                    krate.created_at.as_str(),
                    DATETIME_FORMAT,
                )
                .unwrap();
                let updated_at = chrono::NaiveDateTime::parse_from_str(
                    krate.updated_at.as_str(),
                    DATETIME_FORMAT,
                )
                .unwrap();
                Ok(SearchResult {
                    name: krate.name,
                    max_version: latest.vers,
                    description: krate.description,
                    downloads: krate.downloads,
                    documentation: krate.documentation,
                    repository: krate.repository,
                    created_at,
                    updated_at,
                })
            })
            .collect::<Result<Vec<SearchResult>, Error>>()?;

        let data = SearchResponse {
            crates,
            meta: SearchMeta { total },
        };
        Ok(utils::response::json(&data))
    });

    transaction.await
}
