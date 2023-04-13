use std::num::NonZeroU32;
use std::time::Duration;

use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use ring::digest as hasher;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::db::models::{NewAuthor, NewSalt};
use crate::db::schema::*;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::response::common;
use crate::State;

const REGISTER_FLASH: &'static str = "register.flash";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum RegisterFlashMessage {
    Error { message: String },
    Success { message: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct RegisterForm {
    pub email: String,
    pub name: String,
    pub password: String,
    pub confirm_password: String,
    pub remember: Option<String>,
}

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    if let Some(author) = req.get_author() {
        let state = req.state().as_ref();
        return common::already_logged_in(state, author);
    }

    let flash_message: Option<RegisterFlashMessage> = req.session().get(REGISTER_FLASH);
    if flash_message.is_some() {
        req.session_mut().remove(REGISTER_FLASH);
    }

    let state = req.state();
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
    Ok(utils::response::html(
        engine.render("account/register", &context)?,
    ))
}

pub(crate) async fn post(mut req: Request<State>) -> tide::Result {
    if req.is_authenticated() {
        return Ok(utils::response::redirect("/account/register"));
    }

    let local = &req.state().frontend.config.auth.local;

    if !local.enabled {
        return utils::response::error_html(
            req.state(),
            None,
            StatusCode::BadRequest,
            "local authentication is not allowed on this instance",
        );
    }

    if !local.allow_registration {
        return utils::response::error_html(
            req.state(),
            None,
            StatusCode::BadRequest,
            "local registration is not allowed on this instance",
        );
    }

    //? Deserialize form data.
    let form: RegisterForm = match req.body_form().await {
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

    //? Are all fields filled-in ?
    if form.email.is_empty()
        || form.name.is_empty()
        || form.password.is_empty()
        || form.confirm_password.is_empty()
    {
        let message = String::from("some fields were left empty.");
        let flash_message = RegisterFlashMessage::Error { message };
        req.session_mut().insert(REGISTER_FLASH, &flash_message)?;
        return Ok(utils::response::redirect("/account/register"));
    }

    //? Does the two passwords match (consistency check) ?
    if form.password != form.confirm_password {
        let message = String::from("the two passwords did not match.");
        let flash_message = RegisterFlashMessage::Error { message };
        req.session_mut().insert(REGISTER_FLASH, &flash_message)?;
        return Ok(utils::response::redirect("/account/register"));
    }

    let state = req.state().clone();
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
            req.session_mut().insert(REGISTER_FLASH, &flash_message)?;
            return Ok(utils::response::redirect("/account/register"));
        }

        //? Decode hex-encoded password hash.
        let decoded_password = match hex::decode(form.password.as_bytes()) {
            Ok(passwd) => passwd,
            Err(_) => {
                let message = String::from("password/salt decoding issue.");
                let flash_message = RegisterFlashMessage::Error { message };
                req.session_mut().insert(REGISTER_FLASH, &flash_message)?;
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
        let author_id = authors::table
            .select(authors::id)
            .filter(authors::email.eq(form.email.as_str()))
            .first::<i64>(conn)?;

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
