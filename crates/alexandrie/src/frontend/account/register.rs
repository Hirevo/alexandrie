use std::num::NonZeroU32;
use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum::Form;
use axum_extra::either::Either;
use axum_extra::response::Html;
use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use ring::digest as hasher;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

use crate::config::AppState;
use crate::db::models::{NewAuthor, NewSalt};
use crate::db::schema::*;
use crate::error::FrontendError;
use crate::utils;
use crate::utils::auth::frontend::Auth;
use crate::utils::response::common;

const REGISTER_FLASH: &'static str = "register.flash";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum RegisterFlashMessage {
    Error { message: String },
    Success { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct RegisterForm {
    pub email: String,
    pub name: String,
    pub password: String,
    pub confirm_password: String,
    pub remember: Option<String>,
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Auth>,
    mut session: Session,
) -> Result<(StatusCode, Html<String>), FrontendError> {
    if let Some(Auth(author)) = maybe_author {
        return common::already_logged_in(state.as_ref(), author);
    }

    let flash_message: Option<RegisterFlashMessage> = session.remove(REGISTER_FLASH)?;

    let engine = &state.frontend.handlebars;
    let auth = &state.frontend.config.auth;

    let local_enabled = auth.local.enabled && auth.local.allow_registration;
    let github_enabled = auth.github.enabled && auth.github.allow_registration;
    let gitlab_enabled = auth.gitlab.enabled && auth.gitlab.allow_registration;
    let has_separator = local_enabled && (github_enabled || gitlab_enabled);

    let none_enabled = !local_enabled && !github_enabled && !gitlab_enabled;

    let context = json!({
        "instance": &state.frontend.config,
        "flash": flash_message,
        "local_enabled": local_enabled,
        "github_enabled": github_enabled,
        "gitlab_enabled": gitlab_enabled,
        "has_separator": has_separator,
        "none_enabled": none_enabled,
    });

    let rendered = engine.render("account/register", &context)?;
    Ok((StatusCode::OK, Html(rendered)))
}

pub(crate) async fn post(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Auth>,
    session: Session,
    Form(form): Form<RegisterForm>,
) -> Result<Either<(StatusCode, Html<String>), Redirect>, FrontendError> {
    if maybe_author.is_some() {
        return Ok(Either::E2(Redirect::to("/account/register")));
    }

    if !state.frontend.config.auth.local.enabled {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "local authentication is not allowed on this instance",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    }

    if !state.frontend.config.auth.local.allow_registration {
        let rendered = utils::response::error_html(
            state.as_ref(),
            None,
            "local registration is not allowed on this instance",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    }

    //? Are all fields filled-in ?
    if form.email.is_empty()
        || form.name.is_empty()
        || form.password.is_empty()
        || form.confirm_password.is_empty()
    {
        let message = String::from("some fields were left empty.");
        let flash_message = RegisterFlashMessage::Error { message };
        session.insert(REGISTER_FLASH, &flash_message)?;
        return Ok(Either::E2(Redirect::to("/account/register")));
    }

    //? Does the two passwords match (consistency check) ?
    if form.password != form.confirm_password {
        let message = String::from("the two passwords did not match.");
        let flash_message = RegisterFlashMessage::Error { message };
        session.insert(REGISTER_FLASH, &flash_message)?;
        return Ok(Either::E2(Redirect::to("/account/register")));
    }

    let state = Arc::clone(&state);
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        //? Does the user already exist ?
        let already_exists = sql::select(sql::exists(
            authors::table.filter(authors::email.eq(form.email.as_str())),
        ))
        .get_result(conn)?;
        if already_exists {
            let message = String::from("an author already exists for this email.");
            let flash_message = RegisterFlashMessage::Error { message };
            session.insert(REGISTER_FLASH, &flash_message)?;
            return Ok(Either::E2(Redirect::to("/account/register")));
        }

        dbg!(&form);
        //? Decode hex-encoded password hash.
        let Ok(decoded_password) = hex::decode(form.password.as_bytes()) else {
            let message = String::from("password/salt decoding issue.");
            let flash_message = RegisterFlashMessage::Error { message };
            session.insert(REGISTER_FLASH, &flash_message)?;
            return Ok(Either::E2(Redirect::to("/account/register")));
        };

        //? Generate the user's authentication salt.
        let decoded_generated_salt = {
            let mut data = [0u8; 16];
            let rng = SystemRandom::new();
            rng.fill(&mut data).unwrap();
            hasher::digest(&hasher::SHA512, data.as_ref())
        };

        //? Derive the hashed password data with PBKDF2 (100_000 rounds).
        let encoded_derived_hash = {
            let mut out = [0u8; hasher::SHA512_OUTPUT_LEN];
            let iteration_count = unsafe { NonZeroU32::new_unchecked(100_000) };
            pbkdf2::derive(
                pbkdf2::PBKDF2_HMAC_SHA512,
                iteration_count,
                decoded_generated_salt.as_ref(),
                decoded_password.as_slice(),
                &mut out,
            );
            hex::encode(out.as_ref())
        };

        //? Insert the new author data.
        let new_author = NewAuthor {
            email: form.email.as_str(),
            name: form.name.as_str(),
            passwd: Some(encoded_derived_hash.as_str()),
            github_id: None,
            gitlab_id: None,
        };
        diesel::insert_into(authors::table)
            .values(new_author)
            .execute(conn)?;

        //? Fetch the newly-inserted author back.
        let author_id: i64 = authors::table
            .select(authors::id)
            .filter(authors::email.eq(form.email.as_str()))
            .first(conn)?;

        //? Store the author's newly-generated authentication salt.
        let encoded_generated_salt = hex::encode(decoded_generated_salt.as_ref());
        let new_salt = NewSalt {
            author_id,
            salt: encoded_generated_salt.as_str(),
        };
        diesel::insert_into(salts::table)
            .values(new_salt)
            .execute(conn)?;

        //? Get the maximum duration of the session.
        let expiry = match form.remember.as_deref() {
            Some("on") => time::Duration::seconds(2_592_000), // 30 days
            _ => time::Duration::seconds(86_400),             // 1 day / 24 hours
        };

        //? Set the user's session.
        session.insert("author.id", author_id)?;
        session.expire_in(expiry);

        Ok(Either::E2(Redirect::to("/")))
    });

    transaction.await
}
