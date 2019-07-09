use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use rocket::http::ContentType;
use rocket::response::{Content, Responder, Stream};
use rocket::State;
use semver::Version;

use crate::db::schema::*;
use crate::db::DbConn;
use crate::error::{AlexError, Error};
use crate::index::Indexer;
use crate::state::AppState;
use crate::storage::Store;

/// Route to download a crate's tarball (used by `cargo build`).
///
/// The response is streamed, for performance and memory footprint reasons.
#[get("/crates/<name>/<version>/download")]
pub(crate) fn route(
    state: State<Arc<Mutex<AppState>>>,
    conn: DbConn,
    name: String,
    version: String,
) -> Result<impl Responder, Error> {
    let version = Version::parse(&version)?;
    let state = state.lock().unwrap();
    state.index().refresh()?;
    let downloads = crates::table
        .select(crates::downloads)
        .filter(crates::name.eq(name.as_str()))
        .first::<u64>(&conn.0)
        .optional()?;
    if let Some(downloads) = downloads {
        diesel::update(crates::table)
            .set(crates::downloads.eq(downloads + 1))
            .execute(&conn.0)?;
        let krate = state.storage().read_crate(&name, version)?;
        Ok(Content(ContentType::Binary, Stream::from(krate)))
    } else {
        Err(Error::from(AlexError::CrateNotFound(name)))
    }
}
