use diesel::prelude::*;
use semver::Version;
use tide::{Body, Context, Response};

use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::storage::Store;
use crate::State;

/// Route to download a crate's tarball (used by `cargo build`).
///
/// The response is streamed, for performance and memory footprint reasons.
pub(crate) async fn get(ctx: Context<State>) -> Result<Response, Error> {
    let name = ctx.param::<String>("name").unwrap();
    let version = ctx.param::<Version>("version").unwrap();

    let state = ctx.state();
    let repo = &state.repo;

    // state.index.refresh()?;

    let transaction = repo.transaction(|conn| {
        //? Fetch the download count for this crate.
        let downloads = crates::table
            .select(crates::downloads)
            .filter(crates::name.eq(name.as_str()))
            .first::<u64>(conn)
            .optional()?;

        if let Some(downloads) = downloads {
            //? Increment this crate's download count.
            diesel::update(crates::table.filter(crates::name.eq(name.as_str())))
                .set(crates::downloads.eq(downloads + 1))
                .execute(conn)?;
            let mut krate = state.storage.read_crate(&name, version)?;
            let mut buf = Vec::new();
            krate.read_to_end(&mut buf)?;
            Ok(http::Response::builder()
                .header("content-type", "application/octet-stream")
                .body(Body::from(buf))
                .unwrap())
        } else {
            Err(Error::from(AlexError::CrateNotFound { name }))
        }
    });

    transaction.await
}
