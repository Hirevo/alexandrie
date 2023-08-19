use std::num::NonZeroU32;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::Json;
use semver::Version;
use serde::{Deserialize, Serialize};

use alexandrie_index::Indexer;

use crate::config::AppState;
use crate::error::ApiError;
use crate::utils;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct APIResponse {
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Suggestion {
    pub name: String,
    pub vers: Version,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryParams {
    pub q: String,
    pub limit: Option<NonZeroU32>,
}

/// Route to search through crates (used by `cargo search`).
pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
) -> Result<Json<APIResponse>, ApiError> {
    let name = utils::canonical_name(params.q);
    let limit = params.limit.map_or(10, |limit| limit.get() as usize);

    log::info!("Suggester : {name} & {limit}");

    let results = state.search.suggest(name, limit)?;
    let suggestions: Vec<Suggestion> = results
        .into_iter()
        .map(|krate| {
            let latest = state.index.latest_record(krate.to_lowercase().as_str())?;
            Ok(Suggestion {
                name: krate,
                vers: latest.vers,
            })
        })
        .collect::<Result<_, ApiError>>()?;

    Ok(Json(APIResponse { suggestions }))
}
