use std::num::NonZeroU32;

use diesel::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use tide::Request;

use alexandrie_index::Indexer;

use crate::db::models::Crate;
use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::utils;
use crate::State;

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
    let state = req.state().clone();
    let db = &state.db;

    //? Fetch the latest index changes.
    // state.index.refresh()?;

    let name = utils::canonical_name(params.q);

    //? Build the search pattern.
    let name_pattern = format!("%{0}%", name.replace('\\', "\\\\").replace('%', "\\%"));

    //? Limit the result count depending on parameters.
    let limit = params.limit.map_or(10, |limit| i64::from(limit.get()));

    //? Fetch results.
    let results = db
        .run(move |conn| {
            crates::table
                .filter(crates::canon_name.like(name_pattern.as_str()))
                .limit(limit)
                .load::<Crate>(conn)
        })
        .await?;

    //? Fetch version information about these crates.
    let suggestions = results
        .into_iter()
        .map(|krate| {
            let latest = state.index.latest_record(krate.name.as_str())?;
            Ok(Suggestion {
                name: krate.name,
                vers: latest.vers,
            })
        })
        .collect::<Result<Vec<Suggestion>, Error>>()?;

    let data = APIResponse { suggestions };
    Ok(utils::response::json(&data))
}
