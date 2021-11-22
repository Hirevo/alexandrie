use std::num::NonZeroU32;
use std::time::Duration;

use diesel::prelude::*;
use json::json;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::db::schema::*;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::response::common;
use crate::State;

const LOGIN_FLASH: &'static str = "login.flash";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum LoginFlashMessage {
    Error { message: String },
    Success { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct LoginForm {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    if let Some(author) = req.get_author() {
        let state = req.state().as_ref();
        return common::already_logged_in(state, author);
    }

    let flash_message: Option<LoginFlashMessage> = req.session().get(LOGIN_FLASH);
    if flash_message.is_some() {
        req.session_mut().remove(LOGIN_FLASH);
    }

    let state = req.state();
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
    Ok(utils::response::html(
        engine.render("account/login", &context)?,
    ))
}

pub(crate) async fn post(mut req: Request<State>) -> tide::Result {
    if req.is_authenticated() {
        return Ok(utils::response::redirect("/"));
    }

    if !req.state().frontend.config.auth.local.enabled {
        return utils::response::error_html(
            req.state(),
            None,
            StatusCode::BadRequest,
            "local authentication is not allowed on this instance",
        );
    }

    //? Deserialize form data.
    let form: LoginForm = match req.body_form().await {
        Ok(form) => form,
        Err(_) => {
            return utils::response::error_html(
                req.state(),
                None,
                StatusCode::BadRequest,
                "could not deseriailize form data",
            );
        }
    };

    let state = req.state().clone();
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        //? Get the users' salt and expected hash.
        let results = salts::table
            .inner_join(authors::table)
            .select((authors::id, salts::salt, authors::passwd))
            .filter(authors::email.eq(form.email.as_str()))
            .first::<(i64, String, Option<String>)>(conn)
            .optional()?;

        //? Does the user exist?
        let (author_id, encoded_salt, encoded_expected_hash) = match results {
            Some((id, salt, Some(passwd))) => (id, salt, passwd),
            _ => {
                let message = String::from("invalid email/password combination.");
                let flash_message = LoginFlashMessage::Error { message };
                req.session_mut().insert(LOGIN_FLASH, &flash_message)?;
                return Ok(utils::response::redirect("/account/login"));
            }
        };

        //? Decode hex-encoded hashes.
        let decode_results = hex::decode(encoded_salt.as_str())
            .and_then(|fst| hex::decode(form.password.as_str()).map(move |snd| (fst, snd)))
            .and_then(|(fst, snd)| {
                hex::decode(encoded_expected_hash.as_str()).map(move |trd| (fst, snd, trd))
            });

        let (decoded_salt, decoded_password, decoded_expected_hash) = match decode_results {
            Ok(results) => results,
            Err(_) => {
                let message = String::from("password/salt decoding issue.");
                let flash_message = LoginFlashMessage::Error { message };
                req.session_mut().insert(LOGIN_FLASH, &flash_message)?;
                return Ok(utils::response::redirect("/account/login"));
            }
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
            req.session_mut().insert(LOGIN_FLASH, &flash_message)?;
            return Ok(utils::response::redirect("/account/login"));
        }

        //? Get the maximum duration of the session.
        let expiry = match form.remember.as_deref() {
            Some("on") => Duration::from_secs(2_592_000), // 30 days
            _ => Duration::from_secs(86_400),             // 1 day / 24 hours
        };

        //? Set the user's session.
        req.session_mut().insert("author.id", author_id)?;
        req.session_mut().expire_in(expiry);

        Ok(utils::response::redirect("/"))
    });

    transaction.await
}
