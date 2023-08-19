use std::num::NonZeroUsize;
use std::sync::Arc;

use axum::extract::Query;
use axum::extract::State;
use axum::Json;
use diesel::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};

use alexandrie_index::Indexer;

use crate::config::AppState;
use crate::db::models::Crate;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::ApiError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SearchResponse {
    pub crates: Vec<SearchResult>,
    pub meta: SearchMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct SearchResult {
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
pub(crate) struct SearchMeta {
    pub total: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryParams {
    pub q: String,
    pub per_page: Option<NonZeroUsize>,
    pub page: Option<NonZeroUsize>,
}

/// Route to search through crates (used by `cargo search`).
pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    let query = params.q;
    let per_page = params
        .per_page
        .map(|v| v.get())
        .unwrap_or(crate::fts::DEFAULT_RESULT_PER_PAGE);
    let page = params.page.map(|v| v.get()).unwrap_or(1) - 1;

    let searcher = &state.search;
    // Run query on tantivy and get total and matching ids
    // Perhaps should use suggest method as it allow to deal with "starts with", but I don't think
    // that's what is expected.
    let (total, ids) = searcher.search(&query, page * per_page, per_page)?;

    let db = &state.db;
    let state = Arc::clone(&state);
    let transaction = db.transaction(move |conn| {
        // Get crate from database
        let mut crates = crates::table
            .filter(crates::id.eq_any(&ids))
            .load::<Crate>(conn)?;

        // Sort database result by relevance since we lost ordering...
        // (the `unwrap_or` call should be unreachable, but if it is reached, it would sort the crate towards the end)
        crates.sort_unstable_by_key(|krate| {
            ids.iter()
                .position(|id| *id == krate.id)
                .unwrap_or(ids.len())
        });

        // Fetch missing informations from index
        let crates = crates
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
            .collect::<Result<Vec<SearchResult>, ApiError>>()?;

        Ok::<_, ApiError>(Json(SearchResponse {
            crates,
            meta: SearchMeta { total },
        }))
    });

    transaction.await.map_err(ApiError::from)
}
