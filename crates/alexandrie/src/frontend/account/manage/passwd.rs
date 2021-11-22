use std::num::NonZeroU32;

use diesel::prelude::*;
use ring::digest as hasher;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::db::schema::*;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

use super::{ManageFlashMessage, ACCOUNT_MANAGE_FLASH};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct ChangePasswordForm {
    pub password: String,
    pub new_password: String,
    pub confirm_password: String,
}

pub(crate) async fn post(mut req: Request<State>) -> tide::Result {
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
            return utils::response::error_html(
                req.state(),
                Some(author),
                StatusCode::BadRequest,
                "could not deseriailize form data",
            );
        }
    };

    //? Are all fields filled-in?
    if form.password.is_empty() || form.new_password.is_empty() || form.confirm_password.is_empty()
    {
        let message = String::from("some fields were left empty.");
        let flash_message = ManageFlashMessage::PasswordChangeError { message };
        req.session_mut()
            .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
        return Ok(utils::response::redirect("/account/manage"));
    }

    //? Does the two passwords match (consistency check)?
    if form.new_password != form.confirm_password {
        let message = String::from("the two passwords did not match.");
        let flash_message = ManageFlashMessage::PasswordChangeError { message };
        req.session_mut()
            .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
        return Ok(utils::response::redirect("/account/manage"));
    }

    let state = req.state().clone();
    let db = &state.db;

    let transaction = db.transaction(move |conn| {
        //? Get the users' salt.
        let encoded_salt = salts::table
            .inner_join(authors::table)
            .select(salts::salt)
            .filter(authors::id.eq(author.id))
            .first::<String>(conn)?;

        let decode_results: Result<_, hex::FromHexError> = (|| {
            let fst = hex::decode(encoded_salt.as_str())?;
            let snd = hex::decode(form.new_password.as_str())?;
            Ok((fst, snd))
        })();

        let (decoded_salt, decoded_desired_password) = match decode_results {
            Ok(results) => results,
            Err(_) => {
                let message = String::from("password/salt decoding issue.");
                let flash_message = ManageFlashMessage::PasswordChangeError { message };
                req.session_mut()
                    .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
                return Ok(utils::response::redirect("/account/manage"));
            }
        };

        //? If the user has a password, check that the actual current password matches the one the user submitted.
        //? (a user may not have a password if they registered using an external mean of authentication, like GitHub or GitLab)
        //? (if the user does not have a password, "changing" a password just sets the password to the provided value)
        if let Some(encoded_expected_hash) = author.passwd.as_deref() {
            //? Decode hex-encoded hashes.
            let decode_results: Result<_, hex::FromHexError> = (|| {
                let fst = hex::decode(form.password.as_str())?;
                let snd = hex::decode(encoded_expected_hash)?;
                Ok((fst, snd))
            })();

            let (decoded_current_password, decoded_expected_hash) =
                match decode_results {
                    Ok(results) => results,
                    Err(_) => {
                        let message = String::from("password/salt decoding issue.");
                        let flash_message = ManageFlashMessage::PasswordChangeError { message };
                        req.session_mut()
                            .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
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
                let message = String::from("invalid current password.");
                let flash_message = ManageFlashMessage::PasswordChangeError { message };
                req.session_mut()
                    .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
                return Ok(utils::response::redirect("/account/manage"));
            }
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

        let message = String::from("the password was successfully changed.");
        let flash_message = ManageFlashMessage::PasswordChangeSuccess { message };
        req.session_mut()
            .insert(ACCOUNT_MANAGE_FLASH, &flash_message)?;
        Ok(utils::response::redirect("/account/manage"))
    });

    transaction.await
}
