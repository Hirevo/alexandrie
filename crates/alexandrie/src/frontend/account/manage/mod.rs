use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};
use tide::Request;

/// Password management routes (eg. "/account/manage/password").
pub mod passwd;
/// Token management routes (eg. "/account/manage/tokens").
pub mod tokens;

use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::frontend::helpers;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::response::common;
use crate::State;

const ACCOUNT_MANAGE_FLASH: &str = "account_manage.flash";

/// The flash message type used to communicate between the `/account/manage/...` pages.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum ManageFlashMessage {
    /// Successful password change message.
    PasswordChangeSuccess { message: String },
    /// Failed password change message.
    PasswordChangeError { message: String },
    /// Successful token generation message.
    TokenGenerationSuccess { message: String },
    /// Successful token revocation message.
    TokenRevocationSuccess { message: String },
    /// Failed token revocation message.
    TokenRevocationError { message: String },
}

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    let author = match req.get_author() {
        Some(author) => author,
        None => {
            let state = req.state().as_ref();
            return common::need_to_login(state);
        }
    };

    let state = req.state().clone();
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
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

        let flash_message: Option<ManageFlashMessage> = req.session().get(ACCOUNT_MANAGE_FLASH);
        if flash_message.is_some() {
            req.session_mut().remove(ACCOUNT_MANAGE_FLASH);
        }

        let state = req.state();
        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": author,
            "instance": &state.frontend.config,
            "author": author,
            "owned_crates_count": helpers::humanize_number(owned_crates_count),
            "open_sessions_count": helpers::humanize_number(open_sessions_count),
            "tokens": tokens,
            "flash": flash_message,
        });
        Ok(utils::response::html(
            engine.render("account/manage", &context)?,
        ))
    });

    transaction.await
}
