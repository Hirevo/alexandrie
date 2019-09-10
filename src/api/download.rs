use diesel::prelude::*;
use semver::Version;
use tide::{Context, Response, Body};

use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::index::Indexer;
use crate::storage::Store;
use crate::config::State;

/// Route to download a crate's tarball (used by `cargo build`).
///
/// The response is streamed, for performance and memory footprint reasons.
pub(crate) async fn route(ctx: Context<State>) -> Result<Response, Error> {
    let name = ctx.param::<String>("name").unwrap();
    let version = ctx.param::<Version>("version").unwrap();

    let state = ctx.state();
    let repo = &state.repo;

    // state.index.refresh()?;

    let downloads = repo.run(|conn| crates::table
        .select(crates::downloads)
        .filter(crates::name.eq(name.as_str()))
        .first::<u64>(&conn)
        .optional()
    ).await?;

    if let Some(downloads) = downloads {
        repo.run(|conn| diesel::update(crates::table.filter(crates::name.eq(name.as_str())))
            .set(crates::downloads.eq(downloads + 1))
            .execute(&conn)).await?;
        let mut krate = state.storage.read_crate(&name, version)?;
        let mut buf = Vec::new();
        krate.read_to_end(&mut buf)?;
        Ok(tide::http::Response::builder().header("content-type", "application/octet-stream").body(Body::from(buf)).unwrap())
    } else {
        Err(Error::from(AlexError::CrateNotFound(name)))
    }
}
