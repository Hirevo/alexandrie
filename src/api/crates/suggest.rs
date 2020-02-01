use std::num::NonZeroU32;

use diesel::prelude::*;
use semver::Version;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::CrateRegistration;
use crate::db::schema::*;

use crate::error::{AlexError, Error};
use crate::index::Indexer;
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
pub(crate) async fn get(req: Request<State>) -> Result<Response, Error> {
    let params = req
        .query::<QueryParams>()
        .map_err(|_| AlexError::MissingQueryParams {
            missing_params: &["q"],
        })?;
    let state = req.state().clone();
    let repo = &state.repo;

    //? Fetch the latest index changes.
    // state.index.refresh()?;

    //? Build the search pattern.
    let name_pattern = format!("%{0}%", params.q.replace('\\', "\\\\").replace('%', "\\%"));

    //? Limit the result count depending on parameters.
    let limit = params.limit.map_or(10, |limit| i64::from(limit.get()));

    //? Fetch results.
    let results = repo
        .run(move |conn| {
            crates::table
                .filter(crates::name.like(name_pattern.as_str()))
                .limit(limit)
                .load::<CrateRegistration>(conn)
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
