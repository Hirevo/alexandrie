use async_std::io;

use diesel::prelude::*;
use semver::Version;
use tide::http::mime;
use tide::{Body, Request, Response, StatusCode};

use alexandrie_storage::Store;

use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::State;

/// Route to download a crate's tarball (used by `cargo build`).
///
/// The response is streamed, for performance and memory footprint reasons.
pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let name = req.param("name")?.to_string();
    let version: Version = req.param("version")?.parse()?;

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
            let mut response = Response::new(StatusCode::Ok);
            response.set_content_type(mime::BYTE_STREAM);
            response.set_body(Body::from_reader(io::Cursor::new(buf), None));
            Ok(response)
        } else {
            Err(Error::from(AlexError::CrateNotFound { name }))
        }
    });

    Ok(transaction.await?)
}
