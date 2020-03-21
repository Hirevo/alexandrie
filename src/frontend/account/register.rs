use std::num::NonZeroU32;

use cookie::Cookie;
use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use ring::digest as hasher;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::{NewAuthor, NewSalt, NewSession};
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
struct RegisterForm {
    pub email: String,
    pub name: String,
    pub password: String,
    pub confirm_password: String,
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
        engine.render("account/register", &context).unwrap(),
    ))
}

pub(crate) async fn post(mut req: Request<State>) -> Result<Response, Error> {
    if req.is_authenticated() {
        return Ok(utils::response::redirect("/account/register"));
    }

    //? Deserialize form data.
    let form: RegisterForm = match req.body_form().await {
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
        //? Are all fields filled-in?
        if form.email.is_empty()
            || form.name.is_empty()
            || form.password.is_empty()
            || form.confirm_password.is_empty()
        {
            let error_msg = String::from("some fields were left empty.");
            req.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/register"));
        }

        //? Does the two passwords match (consistency check)?
        if form.password != form.confirm_password {
            let error_msg = String::from("the two passwords did not match.");
            req.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/register"));
        }

        //? Does the user already exist?
        let already_exists = sql::select(sql::exists(
            authors::table.filter(authors::email.eq(form.email.as_str())),
        ))
        .get_result(conn)?;
        if already_exists {
            let error_msg = String::from("an author already exists for this email.");
            req.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/register"));
        }

        //? Decode hex-encoded password hash.
        let decoded_password = match hex::decode(form.password.as_bytes()) {
            Ok(passwd) => passwd,
            Err(_) => {
                let error_msg = String::from("password/salt decoding issue.");
                req.set_flash_message(FlashMessage::from_json(&error_msg)?);
                return Ok(utils::response::redirect("/account/register"));
            }
        };

        //? Generate the user's authentication salt.
        let decoded_generated_salt = {
            let mut data = [0u8; 16];
            let rng = SystemRandom::new();
            rng.fill(&mut data).unwrap();
            hasher::digest(&hasher::SHA512, data.as_ref())
        };

        //? Derive the hashed password data with PBKDF2 (100'000 rounds).
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
        diesel::insert_into(authors::table)
            .values(NewAuthor {
                email: form.email.as_str(),
                name: form.name.as_str(),
                passwd: encoded_derived_hash.as_str(),
            })
            .execute(conn)?;

        //? Fetch the newly-inserted author back.
        let author_id = authors::table
            .select(authors::id)
            .filter(authors::email.eq(form.email.as_str()))
            .first::<i64>(conn)?;

        //? Store the author's newly-generated authentication salt.
        let encoded_generated_salt = hex::encode(decoded_generated_salt.as_ref());
        diesel::insert_into(salts::table)
            .values(NewSalt {
                author_id,
                salt: encoded_generated_salt.as_str(),
            })
            .execute(conn)?;

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
