use std::sync::Arc;

use axum::extract::{Path, State};
use bytes::Bytes;
use diesel::prelude::*;
use semver::Version;

use alexandrie_storage::Store;

use crate::config::AppState;
use crate::db::schema::*;
use crate::error::{AlexError, ApiError};
use crate::utils;

/// Route to download a crate's tarball (used by `cargo build`).
///
/// The response is streamed, for performance and memory footprint reasons.
pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Path((name, version)): Path<(String, Version)>,
) -> Result<Bytes, ApiError> {
    let name = utils::canonical_name(name);

    // state.index.refresh()?;

    let db = &state.db;
    let state = Arc::clone(&state);
    let transaction = db.transaction(move |conn| {
        //? Fetch the download count for this crate.
        let crate_info = crates::table
            .select((crates::name, crates::downloads))
            .filter(crates::canon_name.eq(name.as_str()))
            .first::<(String, i64)>(conn)
            .optional()?;

        if let Some((name, downloads)) = crate_info {
            //? Increment this crate's download count.
            diesel::update(crates::table.filter(crates::name.eq(name.as_str())))
                .set(crates::downloads.eq(downloads + 1))
                .execute(conn)?;

            let krate = state.storage.get_crate(&name, version)?;
            Ok(Bytes::from(krate))
        } else {
            Err(ApiError::from(AlexError::CrateNotFound { name }))
        }
    });

    transaction.await.map_err(ApiError::from)
}
