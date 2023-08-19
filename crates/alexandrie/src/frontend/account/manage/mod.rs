use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum_extra::response::Html;
use axum_sessions::extractors::WritableSession;
use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};

/// Password management routes (eg. "/account/manage/password").
pub mod passwd;
/// Token management routes (eg. "/account/manage/tokens").
pub mod tokens;

use crate::config::AppState;
use crate::db::models::{Author, AuthorToken};
use crate::db::schema::*;
use crate::error::FrontendError;
use crate::frontend::helpers;
use crate::utils::response::common;

const ACCOUNT_MANAGE_FLASH: &'static str = "account_manage.flash";

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

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Author>,
    mut session: WritableSession,
) -> Result<(StatusCode, Html<String>), FrontendError> {
    let Some(author) = maybe_author else {
        let state = state.as_ref();
        return common::need_to_login(state);
    };

    let db = &state.db;
    let state = Arc::clone(&state);

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

        let flash_message: Option<ManageFlashMessage> = session.get(ACCOUNT_MANAGE_FLASH);
        if flash_message.is_some() {
            session.remove(ACCOUNT_MANAGE_FLASH);
        }

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

        let rendered = engine.render("account/manage", &context)?;
        Ok((StatusCode::OK, Html(rendered)))
    });

    transaction.await
}
