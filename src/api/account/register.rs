use std::num::NonZeroU32;

use diesel::dsl as sql;
use diesel::prelude::*;
use http::StatusCode;
use ring::digest as hasher;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::{NewAuthor, NewAuthorToken, NewSalt};
use crate::db::schema::*;
use crate::utils;
use crate::{Error, State};

/// Request body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBody {
    /// The account's email.
    pub email: String,
    /// The account's displayable name.
    pub name: String,
    /// The account's password (hashed and salted client-side).
    pub passwd: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// A registry token for the newly created account.
    pub account_token: String,
}

/// Route to register a new account.
pub async fn post(mut req: Request<State>) -> Result<Response, Error> {
    let state = req.state().clone();
    let repo = &state.repo;

    //? Is the author already logged in ?
    let headers = req.headers().clone();
    let author = repo
        .run(move |conn| utils::checks::get_author(conn, &headers))
        .await;
    if author.is_some() {
        return Ok(utils::response::error(
            StatusCode::UNAUTHORIZED,
            "please log out first to register as a new author",
        ));
    }

    //? Parse request body.
    let body: RequestBody = req.body_json().await?;

    let transaction = repo.transaction(move |conn| {
        //? Does the user already exist ?
        let already_exists = sql::select(sql::exists(
            authors::table.filter(authors::email.eq(body.email.as_str())),
        ))
        .get_result(conn)?;
        if already_exists {
            return Ok(utils::response::error(
                StatusCode::FORBIDDEN,
                "an author already exists for this email.",
            ));
        }

        //? Decode hex-encoded password hash.
        let decoded_password = match hex::decode(body.passwd.as_bytes()) {
            Ok(passwd) => passwd,
            Err(_) => {
                return Ok(utils::response::error(
                    StatusCode::BAD_REQUEST,
                    "could not decode hex-encoded password hash",
                ));
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
            email: body.email.as_str(),
            name: body.name.as_str(),
            passwd: encoded_derived_hash.as_str(),
        };
        diesel::insert_into(authors::table)
            .values(new_author)
            .execute(conn)?;

        //? Fetch the newly-inserted author back.
        let author_id = authors::table
            .select(authors::id)
            .filter(authors::email.eq(body.email.as_str()))
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

        //? Generate new registry token.
        let account_token = utils::auth::generate_token();
        let (token, _) = account_token.split_at(25);

        //? Store the new registry token in the database.
        let token_name = String::from("api-1");
        let new_author_token = NewAuthorToken {
            token,
            author_id,
            name: token_name.as_str(),
        };
        diesel::insert_into(author_tokens::table)
            .values(new_author_token)
            .execute(conn)?;

        let response = ResponseBody { account_token };
        Ok(utils::response::json(&response))
    });

    transaction.await
}
