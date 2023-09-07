use std::sync::Arc;

use axum::extract::State;
use axum::response::Redirect;
use axum::Form;
use axum_sessions::extractors::WritableSession;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

/// Token revocation routes (eg. "/account/manage/tokens/5/revoke").
pub mod revoke;

use crate::config::AppState;
use crate::db::models::NewAuthorToken;
use crate::db::schema::*;
use crate::error::FrontendError;
use crate::utils;
use crate::utils::auth::frontend::Auth;

use super::{ManageFlashMessage, ACCOUNT_MANAGE_FLASH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct CreateTokenForm {
    token_name: String,
}

pub(crate) async fn post(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Auth>,
    mut session: WritableSession,
    Form(form): Form<CreateTokenForm>,
) -> Result<Redirect, FrontendError> {
    let Some(Auth(author)) = maybe_author else {
        return Ok(Redirect::to("/account/manage"));
    };

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
        session.insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
        Ok(Redirect::to("/account/manage"))
    });

    transaction.await
}
