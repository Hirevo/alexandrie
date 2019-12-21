use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

/// Password management routes (eg. "/account/manage/password").
pub mod passwd;
/// Token management routes (eg. "/account/manage/tokens").
pub mod tokens;

use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::error::Error;
use crate::frontend::helpers;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::flash::FlashExt;
use crate::utils::response::common;
use crate::State;

/// The flash message type used to communicate between the `/account/manage/...` pages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type", content = "data")]
pub enum ManageFlashError {
    /// Successful password change message.
    PasswordChangeSuccess(String),
    /// Failed password change message.
    PasswordChangeError(String),
    /// Successful token generation message.
    TokenGenerationSuccess(String),
    /// Successful token revocation message.
    TokenRevocationSuccess(String),
    /// Failed token revocation message.
    TokenRevocationError(String),
}

pub(crate) async fn get(mut req: Request<State>) -> Result<Response, Error> {
    let author = match req.get_author() {
        Some(author) => author,
        None => {
            let state = req.state().as_ref();
            let response = common::need_to_login(state);
            return Ok(response);
        }
    };

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        //? Get the number of crates owned by this author.
        let owned_crates_count = crate_authors::table
            .select(sql::count(crate_authors::id))
            .filter(crate_authors::author_id.eq(author.id))
            .first::<i64>(conn)?;

        //? Get the number of currently open sessions by this author.
        let open_sessions_count = sessions::table
            .select(sql::count(sessions::id))
            .filter(sessions::author_id.eq(author.id))
            .first::<i64>(conn)?;

        //? Get this author's tokens.
        let tokens = author_tokens::table
            .filter(author_tokens::author_id.eq(author.id))
            .load::<AuthorToken>(conn)?;

        let error_msg = req
            .get_flash_message()
            .and_then(|msg| msg.parse_json::<ManageFlashError>().ok());

        #[rustfmt::skip]
        let (
            passwd_change_success_msg,
            passwd_change_error_msg,
            token_gen_success_msg,
            token_revoke_success_msg,
            token_revoke_error_msg,
        ) = match error_msg {
            Some(ManageFlashError::PasswordChangeSuccess(msg)) => (Some(msg), None, None, None, None),
            Some(ManageFlashError::PasswordChangeError(msg)) => (None, Some(msg), None, None, None),
            Some(ManageFlashError::TokenGenerationSuccess(msg)) => (None, None, Some(msg), None, None),
            Some(ManageFlashError::TokenRevocationSuccess(msg)) => (None, None, None, Some(msg), None),
            Some(ManageFlashError::TokenRevocationError(msg)) => (None, None, None, None, Some(msg)),
            None => (None, None, None, None, None),
        };

        let state = req.state();
        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": author,
            "instance": &state.frontend.config,
            "author": author,
            "owned_crates_count": helpers::humanize_number(owned_crates_count),
            "open_sessions_count": helpers::humanize_number(open_sessions_count),
            "tokens": tokens,
            "passwd_change_success_msg": passwd_change_success_msg,
            "passwd_change_error_msg": passwd_change_error_msg,
            "token_generation_success_msg": token_gen_success_msg,
            "token_revocation_success_msg": token_revoke_success_msg,
            "token_revocation_error_msg": token_revoke_error_msg,
        });
        Ok(utils::response::html(
            engine.render("account/manage", &context).unwrap(),
        ))
    });

    transaction.await
}
