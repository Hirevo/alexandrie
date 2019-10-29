use http::status::StatusCode;
use json::json;
use semver::Version;
use tide::{Context, Response};

use crate::error::{AlexError, Error};
use crate::index::Indexer;
use crate::utils;
use crate::State;

pub(crate) async fn put(ctx: Context<State>) -> Result<Response, Error> {
    let name = ctx.param::<String>("name").unwrap();
    let version = ctx.param::<Version>("version").unwrap();

    let state = ctx.state();
    let repo = &state.repo;
    let author = state
        .get_author(ctx.headers())
        .await
        .ok_or(AlexError::InvalidToken)?;

    let transaction = repo.transaction(|conn| {
        //? Does this crate exists?
        let exists = utils::checks::crate_exists(conn, name.as_str())?;
        if !exists {
            return Ok(utils::response::error(
                StatusCode::NOT_FOUND,
                format!("no crates named '{0}' could be found", name),
            ));
        }

        //? Is the user an author of this crate?
        let is_author = utils::checks::is_crate_author(conn, name.as_str(), author.id)?;
        if !is_author {
            return Ok(utils::response::error(
                StatusCode::FORBIDDEN,
                "you are not an author of this crate",
            ));
        }

        state.index.unyank_record(name.as_str(), version.clone())?;

        let msg = format!("Unyanking crate `{0}#{1}`", name.as_str(), version);
        state.index.commit_and_push(msg.as_str())?;

        Ok(tide::response::json(json!({
            "ok": true
        })))
    });

    transaction.await
}
