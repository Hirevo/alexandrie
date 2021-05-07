use diesel::prelude::*;
use tide::Request;

use crate::db::schema::*;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

use super::{ManageFlashMessage, ACCOUNT_MANAGE_FLASH};

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    let author = match req.get_author() {
        Some(author) => author,
        None => {
            return Ok(utils::response::redirect("/account/manage"));
        }
    };

    let id: i64 = req.param("token-id")?.parse()?;

    let state = req.state().clone();
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
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

                let message = String::from("the token has successfully been revoked.");
                let flash_message = ManageFlashMessage::TokenRevocationSuccess { message };
                req.session_mut()
                    .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
                Ok(utils::response::redirect("/account/manage"))
            }
            Some(_) | None => {
                let message = String::from("invalid token to revoke.");
                let flash_message = ManageFlashMessage::TokenRevocationError { message };
                req.session_mut()
                    .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
                Ok(utils::response::redirect("/account/manage"))
            }
        }
    });

    transaction.await
}
