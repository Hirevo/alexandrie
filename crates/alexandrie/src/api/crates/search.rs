use std::num::NonZeroUsize;

use diesel::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use tide::Request;

use alexandrie_index::Indexer;

use crate::db::models::Crate;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::{AlexError, Error};
use crate::utils;
use crate::State;

/// Default number of result per page
/// Perhaps should make this configurable in toml.
const DEFAULT_PER_PAGE: usize = 15;

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
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    pub q: String,
    pub per_page: Option<NonZeroUsize>,
    pub page: Option<NonZeroUsize>,
}

/// Route to search through crates (used by `cargo search`).
pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let params = req
        .query::<QueryParams>()
        .map_err(|_| AlexError::MissingQueryParams {
            missing_params: &["q"],
        })?;
    let state = req.state().clone();

    let query = params.q;
    let per_page = params.per_page.map(|v| v.get()).unwrap_or(DEFAULT_PER_PAGE);
    let page = params.page.map(|v| v.get()).unwrap_or(1) - 1;

    let searcher = &state.search;
    // Run query on tantivy and get total and matching ids
    // Perhaps should use suggest method as it allow to deal with "starts with", but I don't think
    // that's what is expected.
    let (total, ids) = searcher.search(&query, page * per_page, per_page)?;

    let db = &state.db;
    let transaction = db.transaction(move |conn| {
        let state = req.state();

        // Get crate from database
        let results = crates::table
            .filter(crates::id.eq_any(ids))
            .load::<Crate>(conn)?;

        // Fetch missing informations from index
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
