use std::sync::Arc;

use axum::extract::{Path, State};
use axum::Json;
use serde::{Deserialize, Serialize};

use alexandrie_index::{ConfigFile, Indexer};

use crate::config::AppState;
use crate::error::ApiError;

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathParams {
    /// The crate's name.
    pub fst: String,
    /// The crate's description.
    pub snd: String,
    /// The crate's repository link.
    #[serde(rename = "crate")]
    pub krate: Option<String>,
}

/// Route to sparsly access index entries for a crate.
pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(params): Path<PathParams>,
) -> Result<String, ApiError> {
    let (fst, snd, krate) = match params.krate.as_deref() {
        Some(krate) => (params.fst.as_str(), Some(params.snd.as_str()), krate),
        None => (params.fst.as_str(), None, params.snd.as_str()),
    };

    let maybe_expected = match krate.len() {
        0 => None,
        1 => Some(("1", None)),
        2 => Some(("2", None)),
        3 => Some(("3", Some(&krate[..1]))),
        _ => Some((&krate[..2], Some(&krate[2..4]))),
    };

    if maybe_expected.map_or(true, |it| it != (fst, snd)) {
        return Err(ApiError::msg("the crate could not be found"));
    }

    let records = state.index.all_records(krate)?;

    let mut output = String::default();
    for record in records {
        let record = json::to_string(&record)?;
        output.push_str(&record);
        output.push('\n');
    }

    Ok(output)
}

/// Route to sparsly access the index's configuration.
pub async fn get_config(State(state): State<Arc<AppState>>) -> Result<Json<ConfigFile>, ApiError> {
    let config = state.index.configuration()?;
    Ok(Json(config))
}
