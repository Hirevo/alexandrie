use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};
use rocket::{Data, State};
use rocket_contrib::json::Json;
use semver::Version;
use serde::{Deserialize, Serialize};

use crate::{AlexError, AppState, Crate, Error};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResponse {
    pub crates: Vec<SearchResult>,
    pub meta: SearchMeta,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchResult {
    pub name: String,
    pub max_version: Version,
    pub description: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchMeta {
    pub total: u32,
}

#[put("/crates/new", data = "<data>")]
pub fn api_publish(state: State<AppState>, data: Data) -> Result<(), Error> {
    let mut stream = data.open();
    let metadata_size = stream.read_u32::<LittleEndian>()?;
    let mut metadata_bytes = vec![0u8; metadata_size as usize];
    stream.read_exact(metadata_bytes.as_mut_slice())?;
    let metadata: Crate = json::from_slice(metadata_bytes.as_slice())?;
    let crate_size = stream.read_u32::<LittleEndian>()?;
    let mut crate_bytes = vec![0u8; crate_size as usize];
    stream.read_exact(crate_bytes.as_mut_slice())?;

    Ok(())
}

#[get("/crates?<q>&<per_page>")]
pub fn api_search(
    state: State<AppState>,
    q: String,
    per_page: Option<u32>,
) -> Result<Json<SearchResponse>, Error> {
    let state = state.lock().unwrap();
    let found = dbg!(state.search_crates(q.as_str(), per_page))?;
    let total = found.len() as u32;
    let crates = found
        .into_iter()
        .map(|krate| {
            let Crate { name, vers, .. } = krate;
            SearchResult {
                name: name.clone(),
                max_version: vers,
                description: String::default(),
            }
        })
        .collect();
    Ok(Json(SearchResponse {
        crates,
        meta: SearchMeta { total },
    }))
}
