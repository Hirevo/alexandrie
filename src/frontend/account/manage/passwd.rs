use std::num::NonZeroU32;

use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use ring::digest as hasher;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use tide::cookies::ContextExt as CookieExt;
use tide::forms::ContextExt as FormExt;
use tide::{Context, Response};

use crate::db::models::AuthorToken;
use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::frontend::helpers;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::flash::{FlashExt, FlashMessage};
use crate::State;

use super::ManageFlashError;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ChangePasswordForm {
    pub password: String,
    pub new_password: String,
    pub confirm_password: String,
}

pub(crate) async fn post(mut ctx: Context<State>) -> Result<Response, Error> {
    let author = match ctx.get_author() {
        Some(author) => author,
        None => {
            return Ok(utils::response::redirect("/account/manage"));
        }
    };

    // TODO: remove this `unwrap` ASAP!
    let form: ChangePasswordForm = ctx.body_form().await.unwrap();

    let state = ctx.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(|conn| {
        //? Are all fields filled-in?
        if form.password.is_empty()
            || form.new_password.is_empty()
            || form.confirm_password.is_empty()
        {
            let error_msg =
                ManageFlashError::PasswordError(String::from("some fields were left empty."));
            ctx.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/manage"));
        }

        //? Does the two passwords match (consistency check)?
        if form.new_password != form.confirm_password {
            let error_msg =
                ManageFlashError::PasswordError(String::from("the two passwords did not match."));
            ctx.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/manage"));
        }

        //? Get the users' salt and expected hash.
        let encoded_salt = salts::table
            .inner_join(authors::table)
            .select(salts::salt)
            .filter(authors::id.eq(author.id))
            .first::<String>(conn)?;

        //? Decode hex-encoded hashes.
        // TODO: remove these `unwrap` ASAP!
        let decoded_salt = hex::decode(encoded_salt).unwrap();
        let decoded_current_password = hex::decode(form.password.as_bytes()).unwrap();
        let decoded_desired_password = hex::decode(form.new_password.as_bytes()).unwrap();
        let decoded_expected_hash = hex::decode(author.passwd.as_bytes()).unwrap();

        //? Verify client password against the expected hash (through PBKDF2).
        let password_match = {
            let iteration_count = unsafe { NonZeroU32::new_unchecked(100_000) };
            let outcome = pbkdf2::verify(
                pbkdf2::PBKDF2_HMAC_SHA512,
                iteration_count,
                decoded_salt.as_slice(),
                decoded_current_password.as_slice(),
                decoded_expected_hash.as_slice(),
            );
            outcome.is_ok()
        };

        if !password_match {
            let error_msg =
                ManageFlashError::PasswordError(String::from("invalid current password."));
            ctx.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/manage"));
        }

        //? Derive the hashed password data with PBKDF2 (100'000 rounds).
        let encoded_derived_hash = {
            let mut out = [0u8; hasher::SHA512_OUTPUT_LEN];
            let iteration_count = unsafe { NonZeroU32::new_unchecked(100_000) };
            pbkdf2::derive(
                pbkdf2::PBKDF2_HMAC_SHA512,
                iteration_count,
                decoded_salt.as_slice(),
                decoded_desired_password.as_slice(),
                &mut out,
            );
            hex::encode(out.as_ref())
        };

        diesel::update(authors::table)
            .set(authors::passwd.eq(encoded_derived_hash.as_str()))
            .execute(conn)?;

        let success_msg = ManageFlashError::PasswordSuccess(String::from(
            "the password was successfully changed.",
        ));
        ctx.set_flash_message(FlashMessage::from_json(&success_msg)?);
        Ok(utils::response::redirect("/account/manage"))
    });

    transaction.await
}
