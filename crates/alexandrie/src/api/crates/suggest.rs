use std::num::NonZeroU32;

use log::info;
use semver::Version;
use serde::{Deserialize, Serialize};
use tide::Request;

use alexandrie_index::Indexer;

use crate::error::{AlexError, Error};
use crate::State;
use crate::utils;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct APIResponse {
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct Suggestion {
    pub name: String,
    pub vers: Version,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct QueryParams {
    pub q: String,
    pub limit: Option<NonZeroU32>,
}

/// Route to search through crates (used by `cargo search`).
pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let params = req
        .query::<QueryParams>()
        .map_err(|_| AlexError::MissingQueryParams {
            missing_params: &["q"],
        })?;

    let name = utils::canonical_name(params.q);
    let limit = params.limit.map_or(10, |limit| limit.get() as usize);

    info!("Suggester : {} & {}", name, limit);
    let state = req.state().clone();
    let index = &state.index;

    let results = {
        let tantivy = (&state.search)
            .read()
            .map_err(|error| Error::PoisonedError(error.to_string()))?;
        tantivy.suggest(name, limit)?
    };
    let suggestions = results
        .into_iter()
        .map(|krate| {
            let latest = index.latest_record(krate.to_lowercase().as_str())?;
            Ok(Suggestion {
                name: krate,
                vers: latest.vers,
            })
        })
        .collect::<Result<Vec<Suggestion>, Error>>()?;

    let data = APIResponse { suggestions };
    Ok(utils::response::json(&data))
}
