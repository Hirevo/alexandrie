use std::num::NonZeroU32;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::Form;
use axum_extra::either::Either;
use axum_extra::response::Html;
use axum_sessions::extractors::WritableSession;
use diesel::prelude::*;
use json::json;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};

use crate::config::AppState;
use crate::db::schema::*;
use crate::error::FrontendError;
use crate::utils;
use crate::utils::auth::frontend::Auth;
use crate::utils::response::common;

const LOGIN_FLASH: &'static str = "login.flash";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum LoginFlashMessage {
    Error { message: String },
    Success { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct LoginForm {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Auth>,
    mut session: WritableSession,
) -> Result<(StatusCode, Html<String>), FrontendError> {
    if let Some(Auth(author)) = maybe_author {
        return common::already_logged_in(state.as_ref(), author);
    }

    let flash_message: Option<LoginFlashMessage> = session.get(LOGIN_FLASH);
    if flash_message.is_some() {
        session.remove(LOGIN_FLASH);
    }

    let engine = &state.frontend.handlebars;
    let auth = &state.frontend.config.auth;

    let local_enabled = auth.local.enabled;
    let github_enabled = auth.github.enabled;
    let gitlab_enabled = auth.gitlab.enabled;
    let local_registration_enabled = auth.local.allow_registration;
    let has_separator = local_enabled && (github_enabled || gitlab_enabled);

    let context = json!({
        "instance": &state.frontend.config,
        "flash": flash_message,
        "local_enabled": local_enabled,
        "github_enabled": github_enabled,
        "gitlab_enabled": gitlab_enabled,
        "local_registration_enabled": local_registration_enabled,
        "has_separator": has_separator,
    });

    let rendered = engine.render("account/login", &context)?;
    Ok((StatusCode::OK, Html(rendered)))
}

pub(crate) async fn post(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Auth>,
    mut session: WritableSession,
    Form(form): Form<LoginForm>,
) -> Result<Either<(StatusCode, Html<String>), Redirect>, FrontendError> {
    if maybe_author.is_some() {
        return Ok(Either::E2(Redirect::to("/")));
    }

    if !state.frontend.config.auth.local.enabled {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "local authentication is not allowed on this instance",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    }

    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        //? Get the users' salt and expected hash.
        let maybe_results: Option<(i64, String, Option<String>)> = salts::table
            .inner_join(authors::table)
            .select((authors::id, salts::salt, authors::passwd))
            .filter(authors::email.eq(form.email.as_str()))
            .first(conn)
            .optional()?;

        //? Does the user exist?
        let (author_id, encoded_salt, encoded_expected_hash) = match maybe_results {
            Some((id, salt, Some(passwd))) => (id, salt, passwd),
            _ => {
                let message = String::from("invalid email/password combination.");
                let flash_message = LoginFlashMessage::Error { message };
                session.insert(LOGIN_FLASH, &flash_message)?;
                return Ok(Either::E2(Redirect::to("/account/login")));
            }
        };

        //? Decode hex-encoded hashes.
        let maybe_results = hex::decode(encoded_salt.as_str())
            .and_then(|fst| hex::decode(form.password.as_str()).map(move |snd| (fst, snd)))
            .and_then(|(fst, snd)| {
                hex::decode(encoded_expected_hash.as_str()).map(move |trd| (fst, snd, trd))
            });

        let Ok((decoded_salt, decoded_password, decoded_expected_hash)) = maybe_results else {
            let message = String::from("password/salt decoding issue.");
            let flash_message = LoginFlashMessage::Error { message };
            session.insert(LOGIN_FLASH, &flash_message)?;
            return Ok(Either::E2(Redirect::to("/account/login")));
        };

        //? Verify client password against the expected hash (through PBKDF2).
        let password_match = {
            let iteration_count = unsafe { NonZeroU32::new_unchecked(100_000) };
            let outcome = pbkdf2::verify(
                pbkdf2::PBKDF2_HMAC_SHA512,
                iteration_count,
                decoded_salt.as_slice(),
                decoded_password.as_slice(),
                decoded_expected_hash.as_slice(),
            );
            outcome.is_ok()
        };

        if !password_match {
            let message = String::from("invalid email/password combination.");
            let flash_message = LoginFlashMessage::Error { message };
            session.insert(LOGIN_FLASH, &flash_message)?;
            return Ok(Either::E2(Redirect::to("/account/login")));
        }

        //? Get the maximum duration of the session.
        let expiry = match form.remember.as_deref() {
            Some("on") => Duration::from_secs(2_592_000), // 30 days
            _ => Duration::from_secs(86_400),             // 1 day / 24 hours
        };

        //? Set the user's session.
        session.insert("author.id", author_id)?;
        session.expire_in(expiry);

        Ok(Either::E2(Redirect::to("/")))
    });

    transaction.await
}
