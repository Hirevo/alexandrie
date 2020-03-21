use std::num::NonZeroU32;

use cookie::Cookie;
use diesel::prelude::*;
use json::json;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::NewSession;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::Error;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::cookies::CookiesExt;
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

pub(crate) async fn get(mut req: Request<State>) -> Result<Response, Error> {
    if let Some(author) = req.get_author() {
        let state = req.state().as_ref();
        let response = common::already_logged_in(state, author);
        return Ok(response);
    }

    let error_msg = req
        .get_flash_message()
        .and_then(|msg| msg.parse_json::<String>().ok());
    let state = req.state();
    let engine = &state.frontend.handlebars;
    let context = json!({
        "instance": &state.frontend.config,
        "error_msg": error_msg,
    });
    Ok(utils::response::html(
        engine.render("account/login", &context).unwrap(),
    ))
}

pub(crate) async fn post(mut req: Request<State>) -> Result<Response, Error> {
    if req.is_authenticated() {
        return Ok(utils::response::redirect("/"));
    }

    //? Deserialize form data.
    let form: LoginForm = match req.body_form().await {
        Ok(form) => form,
        Err(_) => {
            return Ok(utils::response::error_html(
                req.state(),
                None,
                http::StatusCode::BAD_REQUEST,
                "could not deseriailize form data",
            ));
        }
    };

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        //? Get the users' salt and expected hash.
        let results = salts::table
            .inner_join(authors::table)
            .select((authors::id, salts::salt, authors::passwd))
            .filter(authors::email.eq(form.email.as_str()))
            .first::<(i64, String, String)>(conn)
            .optional()?;

        //? Does the user exist?
        let (author_id, encoded_salt, encoded_expected_hash) = match results {
            Some(results) => results,
            None => {
                let error_msg = String::from("invalid email/password combination.");
                req.set_flash_message(FlashMessage::from_json(&error_msg)?);
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
                let error_msg = String::from("password/salt decoding issue.");
                req.set_flash_message(FlashMessage::from_json(&error_msg)?);
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
            let error_msg = String::from("invalid email/password combination.");
            req.set_flash_message(FlashMessage::from_json(&error_msg)?);
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
                expires: (chrono::Utc::now() + max_age)
                    .naive_utc()
                    .format(DATETIME_FORMAT)
                    .to_string(),
            })
            .execute(conn)?;

        //? Build the user's cookie.
        let cookie = Cookie::build(utils::auth::COOKIE_NAME, session_token)
            .path("/")
            .http_only(true)
            .max_age(max_age_cookie)
            .finish();

        //? Set the user's cookie.
        req.set_cookie(cookie).unwrap();

        Ok(utils::response::redirect("/"))
    });

    transaction.await
}
