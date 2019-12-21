use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

/// Token revocation routes (eg. "/account/manage/tokens/5/revoke").
pub mod revoke;

use crate::db::models::NewAuthorToken;
use crate::db::schema::*;
use crate::error::Error;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::flash::{FlashExt, FlashMessage};
use crate::State;

use super::ManageFlashError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CreateTokenForm {
    token_name: String,
}

pub(crate) async fn post(mut req: Request<State>) -> Result<Response, Error> {
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
            return Ok(utils::response::error_html(
                req.state(),
                Some(author),
                http::StatusCode::BAD_REQUEST,
                "could not deseriailize form data",
            ));
        }
    };

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        let token = utils::auth::generate_token();
        let (token, _) = token.split_at(25);

        diesel::insert_into(author_tokens::table)
            .values(NewAuthorToken {
                token,
                name: form.token_name.as_str(),
                author_id: author.id,
            })
            .execute(conn)?;

        let error_msg = ManageFlashError::TokenGenerationSuccess(String::from(token));
        req.set_flash_message(FlashMessage::from_json(&error_msg)?);
        Ok(utils::response::redirect("/account/manage"))
    });

    transaction.await
}
