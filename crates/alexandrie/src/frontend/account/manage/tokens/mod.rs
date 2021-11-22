use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

/// Token revocation routes (eg. "/account/manage/tokens/5/revoke").
pub mod revoke;

use crate::db::models::NewAuthorToken;
use crate::db::schema::*;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

use super::{ManageFlashMessage, ACCOUNT_MANAGE_FLASH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CreateTokenForm {
    token_name: String,
}

pub(crate) async fn post(mut req: Request<State>) -> tide::Result {
    let author = match req.get_author() {
        Some(author) => author,
        None => {
            return Ok(utils::response::redirect("/account/manage"));
        }
    };

    //? Deserialize form data.
    let form: CreateTokenForm = match req.body_form().await {
        Ok(form) => form,
        Err(_) => {
            return utils::response::error_html(
                req.state(),
                Some(author),
                StatusCode::BadRequest,
                "could not deseriailize form data",
            );
        }
    };

    let state = req.state().clone();
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        //? Generate new registry token.
        let token = utils::auth::generate_token();
        let (token, _) = token.split_at(25);

        let new_author_token = NewAuthorToken {
            token,
            name: form.token_name.as_str(),
            author_id: author.id,
        };
        diesel::insert_into(author_tokens::table)
            .values(new_author_token)
            .execute(conn)?;

        let message = String::from(token);
        let flash_message = ManageFlashMessage::TokenGenerationSuccess { message };
        req.session_mut()
            .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
        Ok(utils::response::redirect("/account/manage"))
    });

    transaction.await
}
