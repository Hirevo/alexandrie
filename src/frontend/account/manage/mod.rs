use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};
use tide::cookies::ContextExt;
use tide::{Context, Response};

/// Password management routes (eg. "/account/manage/password").
pub mod passwd;
/// Token management routes (eg. "/account/manage/tokens").
pub mod tokens;

use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::frontend::helpers;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::response::common;
use crate::utils::flash::{FlashExt, FlashMessage};
use crate::State;

/// The flash message type used to communicate between the `/account/manage/...` pages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", tag = "type", content = "data")]
pub enum ManageFlashError {
    /// Successful password change message.
    PasswordSuccess(String),
    /// Failed password change message.
    PasswordError(String),
    /// Successful token generation message.
    TokenSuccess(String),
}

pub(crate) async fn get(mut ctx: Context<State>) -> Result<Response, Error> {
    let author = match ctx.get_author() {
        Some(author) => author,
        None => {
            let state = ctx.state().as_ref();
            let response = common::need_to_login(state);
            return Ok(response);
        }
    };

    let state = ctx.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(|conn| {
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

        let error_msg = ctx
            .get_flash_message()
            .and_then(|msg| msg.parse_json::<ManageFlashError>().ok());
        let (passwd_success_msg, passwd_error_msg, token_success_msg) = match error_msg {
            Some(ManageFlashError::PasswordSuccess(msg)) => (Some(msg), None, None),
            Some(ManageFlashError::PasswordError(msg)) => (None, Some(msg), None),
            Some(ManageFlashError::TokenSuccess(msg)) => (None, None, Some(msg)),
            None => (None, None, None),
        };
        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": author,
            "instance": &state.frontend.config,
            "author": author,
            "owned_crates_count": helpers::humanize_number(owned_crates_count),
            "open_sessions_count": helpers::humanize_number(open_sessions_count),
            "tokens": tokens,
            "passwd_success_msg": passwd_success_msg,
            "passwd_error_msg": passwd_error_msg,
            "token_success_msg": token_success_msg,
        });
        Ok(utils::response::html(
            engine.render("account/manage", &context).unwrap(),
        ))
    });

    transaction.await
}
