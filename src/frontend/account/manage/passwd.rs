use std::num::NonZeroU32;

use diesel::prelude::*;
use ring::digest as hasher;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::schema::*;
use crate::error::Error;
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

pub(crate) async fn post(mut req: Request<State>) -> Result<Response, Error> {
    let author = match req.get_author() {
        Some(author) => author,
        None => {
            return Ok(utils::response::redirect("/account/manage"));
        }
    };

    //? Deserialize form data.
    let form: ChangePasswordForm = match req.body_form().await {
        Ok(form) => form,
        Err(_) => {
            return Ok(utils::response::error_html(
                req.state(),
                Some(author),
                http::StatusCode::BAD_REQUEST,
                "could not deseriailize form data",
            ));
        }
    };

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        //? Are all fields filled-in?
        if form.password.is_empty()
            || form.new_password.is_empty()
            || form.confirm_password.is_empty()
        {
            let error_msg = String::from("some fields were left empty.");
            let error_msg = ManageFlashError::PasswordChangeError(error_msg);
            req.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/manage"));
        }

        //? Does the two passwords match (consistency check)?
        if form.new_password != form.confirm_password {
            let error_msg = String::from("the two passwords did not match.");
            let error_msg = ManageFlashError::PasswordChangeError(error_msg);
            req.set_flash_message(FlashMessage::from_json(&error_msg)?);
            return Ok(utils::response::redirect("/account/manage"));
        }

        //? Get the users' salt and expected hash.
        let encoded_salt = salts::table
            .inner_join(authors::table)
            .select(salts::salt)
            .filter(authors::id.eq(author.id))
            .first::<String>(conn)?;

        //? Decode hex-encoded hashes.
        let decode_results: Result<_, hex::FromHexError> = hex::decode(encoded_salt.as_str())
            .and_then(|fst| hex::decode(form.password.as_str()).map(move |snd| (fst, snd)))
            .and_then(|(fst, snd)| {
                hex::decode(form.new_password.as_str()).map(move |trd| (fst, snd, trd))
            })
            .and_then(|(fst, snd, trd)| {
                hex::decode(author.passwd.as_str()).map(move |frth| (fst, snd, trd, frth))
            });

        let (
            decoded_salt,
            decoded_current_password,
            decoded_desired_password,
            decoded_expected_hash,
        ) = match decode_results {
            Ok(results) => results,
            Err(_) => {
                let error_msg = String::from("password/salt decoding issue.");
                let error_msg = ManageFlashError::PasswordChangeError(error_msg);
                req.set_flash_message(FlashMessage::from_json(&error_msg)?);
                return Ok(utils::response::redirect("/account/manage"));
            }
        };

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
                ManageFlashError::PasswordChangeError(String::from("invalid current password."));
            req.set_flash_message(FlashMessage::from_json(&error_msg)?);
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

        let success_msg = ManageFlashError::PasswordChangeSuccess(String::from(
            "the password was successfully changed.",
        ));
        req.set_flash_message(FlashMessage::from_json(&success_msg)?);
        Ok(utils::response::redirect("/account/manage"))
    });

    transaction.await
}
