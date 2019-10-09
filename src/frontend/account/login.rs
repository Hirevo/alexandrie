use std::num::NonZeroU32;

use cookie::Cookie;
use diesel::prelude::*;
use json::json;
use ring::digest as hasher;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tide::cookies::ContextExt as CookieExt;
use tide::forms::ContextExt as FormExt;
use tide::{Context, Response};

use crate::db::models::NewSession;
use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::flash::{FlashExt, FlashMessage};
use crate::utils::response::common;
use crate::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct LoginForm {
    pub email: String,
    pub password: String,
    pub remember: Option<String>,
}

pub(crate) async fn get(mut ctx: Context<State>) -> Result<Response, Error> {
    if let Some(author) = ctx.get_author() {
        let state = ctx.state().as_ref();
        let response = common::already_logged_in(state, author);
        return Ok(response);
    }

    let error_msg = ctx
        .get_flash_message()
        .and_then(|msg| msg.parse_json::<String>().ok());
    let state = ctx.state();
    let engine = &state.frontend.handlebars;
    let context = json!({
        "instance": &state.frontend.config,
        "error_msg": error_msg,
    });
    Ok(utils::response::html(
        engine.render("account/login", &context).unwrap(),
    ))
}

pub(crate) async fn post(mut ctx: Context<State>) -> Result<Response, Error> {
    if ctx.is_authenticated() {
        return Ok(utils::response::redirect("/"));
    }

    // TODO: remove this `unwrap` ASAP!
    let form: LoginForm = ctx.body_form().await.unwrap();

    let state = ctx.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(|conn| {
        //? Get the users' salt and expected hash.
        let results = salts::table
            .inner_join(authors::table)
            .select((authors::id, salts::salt, authors::passwd))
            .filter(authors::email.eq(form.email.as_str()))
            .first::<(u64, String, String)>(conn)
            .optional()?;

        //? Does the user exist?
        let (author_id, encoded_salt, encoded_expected_hash) = match results {
            Some(results) => results,
            None => {
                let error_msg = String::from("invalid email/password combination.");
                ctx.set_flash_message(FlashMessage::from_json(&error_msg)?);
                return Ok(utils::response::redirect("/account/login"));
            }
        };

        //? Decode hex-encoded hashes.
        // TODO: remove these `unwrap` ASAP!
        let decoded_salt = hex::decode(encoded_salt).unwrap();
        let decoded_password = hex::decode(form.password.as_bytes()).unwrap();
        let decoded_expected_hash = hex::decode(encoded_expected_hash).unwrap();

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
            let error_msg = String::from("invalid email/password combination.");
            ctx.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/login"));
        }

        //? Generate a new session token.
        let session_token = utils::auth::generate_token();

        //? Get the maximum duration of the session.
        let (max_age, max_age_cookie) = match form.remember.as_deref() {
            Some("on") => (chrono::Duration::days(30), time::Duration::days(30)),
            _ => (chrono::Duration::days(1), time::Duration::days(1)),
        };

        //? Insert the newly-created session.
        diesel::insert_into(sessions::table)
            .values(NewSession {
                author_id,
                token: session_token.as_str(),
                expires: (chrono::Utc::now() + max_age).naive_utc(),
            })
            .execute(conn)?;

        //? Build the user's cookie.
        let cookie = Cookie::build(utils::auth::COOKIE_NAME, session_token)
            .path("/")
            .http_only(true)
            .max_age(max_age_cookie)
            .finish();

        //? Set the user's cookie.
        ctx.set_cookie(cookie).unwrap();

        Ok(utils::response::redirect("/"))
    });

    transaction.await
}
