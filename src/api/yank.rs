use http::status::StatusCode;
use json::json;
use semver::Version;
use tide::{Request, Response};

use crate::error::{AlexError, Error};
use crate::index::Indexer;
use crate::utils;
use crate::State;

pub(crate) async fn delete(req: Request<State>) -> Result<Response, Error> {
    let name = req.param::<String>("name").unwrap();
    let version = req.param::<Version>("version").unwrap();

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        let state = req.state();

        let author =
            utils::checks::get_author(conn, req.headers()).ok_or(AlexError::InvalidToken)?;

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

        state.index.yank_record(name.as_str(), version.clone())?;

        let msg = format!("Yanking crate `{0}#{1}`", name.as_str(), version);
        state.index.commit_and_push(msg.as_str())?;

        let data = json!({
            "ok": true
        });
        Ok(utils::response::json(&data))
    });

    transaction.await
}
