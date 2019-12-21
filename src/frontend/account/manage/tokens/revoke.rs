use diesel::prelude::*;
use tide::{Request, Response};

use crate::db::schema::*;
use crate::error::Error;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::flash::{FlashExt, FlashMessage};
use crate::State;

use super::ManageFlashError;

pub(crate) async fn get(mut req: Request<State>) -> Result<Response, Error> {
    let author = match req.get_author() {
        Some(author) => author,
        None => {
            return Ok(utils::response::redirect("/account/manage"));
        }
    };

    let id = req.param::<i64>("token-id").unwrap();

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        let token_author_id = author_tokens::table
            .select(author_tokens::author_id)
            .filter(author_tokens::id.eq(id))
            .first::<i64>(conn)
            .optional()?;

        match token_author_id {
            Some(token_author_id) if token_author_id == author.id => {
                diesel::delete(
                    author_tokens::table
                        .filter(author_tokens::id.eq(id))
                        .filter(author_tokens::author_id.eq(author.id)),
                )
                .execute(conn)?;

                let error_msg = String::from("the token has successfully been revoked.");
                let error_msg = ManageFlashError::TokenRevocationSuccess(error_msg);
                req.set_flash_message(FlashMessage::from_json(&error_msg)?);
                Ok(utils::response::redirect("/account/manage"))
            }
            Some(_) | None => {
                let error_msg = String::from("invalid token to revoke.");
                let error_msg = ManageFlashError::TokenRevocationError(error_msg);
                req.set_flash_message(FlashMessage::from_json(&error_msg)?);
                Ok(utils::response::redirect("/account/manage"))
            }
        }
    });

    transaction.await
}
