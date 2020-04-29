use async_std::io;

use diesel::prelude::*;
use semver::Version;
use tide::{Request, Response};

use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::storage::Store;
use crate::State;

/// Route to download a crate's tarball (used by `cargo build`).
///
/// The response is streamed, for performance and memory footprint reasons.
pub(crate) async fn get(req: Request<State>) -> Result<Response, Error> {
    let name = req.param::<String>("name").unwrap();
    let version = req.param::<Version>("version").unwrap();

    let state = req.state().clone();
    let repo = &state.repo;

    // state.index.refresh()?;

    let transaction = repo.transaction(move |conn| {
        let state = req.state();

        //? Fetch the download count for this crate.
        let downloads = crates::table
            .select(crates::downloads)
            .filter(crates::name.eq(name.as_str()))
            .first::<i64>(conn)
            .optional()?;

        if let Some(downloads) = downloads {
            //? Increment this crate's download count.
            diesel::update(crates::table.filter(crates::name.eq(name.as_str())))
                .set(crates::downloads.eq(downloads + 1))
                .execute(conn)?;
            let mut krate = state.storage.read_crate(&name, version)?;
            let mut buf = Vec::new();
            krate.read_to_end(&mut buf)?;
            Ok(Response::new(200)
                .set_header("content-type", "application/octet-stream")
                .body(io::Cursor::new(buf)))
        } else {
            Err(Error::from(AlexError::CrateNotFound { name }))
        }
    });

    transaction.await
}
