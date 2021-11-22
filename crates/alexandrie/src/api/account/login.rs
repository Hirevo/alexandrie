use std::num::NonZeroU32;

use diesel::prelude::*;
use ring::digest as hasher;
use ring::pbkdf2;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::db::models::NewAuthorToken;
use crate::db::schema::*;
use crate::utils;
use crate::State;

/// Request body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBody {
    /// The account's email.
    pub email: String,
    /// The account's password.
    pub passwd: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// A registry token for that account.
    pub token: String,
}

/// Route to log in to an account.
pub async fn post(mut req: Request<State>) -> tide::Result {
    let state = req.state().clone();
    let db = &state.db;

    //? Is the author logged in ?
    let author = if let Some(headers) = req.header(utils::auth::AUTHORIZATION_HEADER) {
        let header = headers.last().to_string();
        db.run(move |conn| utils::checks::get_author(conn, header))
            .await
    } else {
        None
    };
    if author.is_some() {
        return Ok(utils::response::error(
            StatusCode::Unauthorized,
            "please log out first to register as a new author",
        ));
    }

    //? Parse request body.
    let body: RequestBody = req.body_json().await?;

    let transaction = db.transaction(move |conn| {
        //? Get the users' salt and expected hash.
        let results = salts::table
            .inner_join(authors::table)
            .select((authors::id, salts::salt, authors::passwd))
            .filter(authors::email.eq(body.email.as_str()))
            .first::<(i64, String, Option<String>)>(conn)
            .optional()?;

        //? Does the user exist?
        let (author_id, encoded_salt, encoded_expected_hash) = match results {
            Some((author_id, salt, Some(passwd))) => (author_id, salt, passwd),
            _ => {
                return Ok(utils::response::error(
                    StatusCode::Forbidden,
                    "invalid email/password combination.",
                ));
            }
        };

        //? Decode hex-encoded hashes.
        let decode_results = hex::decode(encoded_salt.as_str())
            .and_then(|fst| hex::decode(encoded_expected_hash.as_str()).map(move |snd| (fst, snd)));

        let (decoded_salt, decoded_expected_hash) = match decode_results {
            Ok(results) => results,
            Err(_) => {
                return Ok(utils::response::error(
                    StatusCode::InternalServerError,
                    "an author already exists for this email.",
                ));
            }
        };

        //? First rounds of PBKDF2 (5_000 rounds, it corresponds to what the frontend does, cf. `wasm-pbkdf2` sub-crate).
        let hashed_passwd = {
            let mut out = [0u8; hasher::SHA512_OUTPUT_LEN];
            let iteration_count = unsafe { NonZeroU32::new_unchecked(5_000) };
            pbkdf2::derive(
                pbkdf2::PBKDF2_HMAC_SHA512,
                iteration_count,
                body.email.as_bytes(),
                body.passwd.as_bytes(),
                &mut out,
            );
            out
        };

        //? Verify client password against the expected hash (through PBKDF2).
        let password_match = {
            let iteration_count = unsafe { NonZeroU32::new_unchecked(100_000) };
            let outcome = pbkdf2::verify(
                pbkdf2::PBKDF2_HMAC_SHA512,
                iteration_count,
                decoded_salt.as_slice(),
                hashed_passwd.as_ref(),
                decoded_expected_hash.as_slice(),
            );
            outcome.is_ok()
        };

        if !password_match {
            return Ok(utils::response::error(
                StatusCode::Forbidden,
                "invalid email/password combination.",
            ));
        }

        //? Generate new registry token.
        let account_token = utils::auth::generate_token();
        let (token, _) = account_token.split_at(25);

        //? Store the new registry token in the database.
        let new_author_token = NewAuthorToken {
            name: "API",
            token,
            author_id,
        };

        //? Try to insert, but it might fail if one already exists.
        //? (note the absence of the `?` operator here)
        //? That's OK, we'll just reuse it then.
        let _ = diesel::insert_into(author_tokens::table)
            .values(new_author_token)
            .execute(conn);

        //? Get back that token (or the already existing one).
        let token: String = author_tokens::table
            .filter(author_tokens::name.eq("API"))
            .filter(author_tokens::author_id.eq(author_id))
            .select(author_tokens::token)
            .first(conn)?;

        let response = ResponseBody { token };
        Ok(utils::response::json(&response))
    });

    transaction.await
}
