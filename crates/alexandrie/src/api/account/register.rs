use std::num::NonZeroU32;

use diesel::dsl as sql;
use diesel::prelude::*;
use ring::digest as hasher;
use ring::pbkdf2;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};

use crate::db::models::{NewAuthor, NewAuthorToken, NewSalt};
use crate::db::schema::*;
use crate::utils;
use crate::State;

/// Request body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestBody {
    /// The account's email.
    pub email: String,
    /// The account's displayable name.
    pub name: String,
    /// The account's password.
    pub passwd: String,
}

/// Response body for this route.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseBody {
    /// A registry token for the newly created account.
    pub token: String,
}

/// Route to register a new account.
pub async fn post(mut req: Request<State>) -> tide::Result {
    let state = req.state().clone();
    let db = &state.db;

    //? Is the author already logged in ?
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
        //? Does the user already exist ?
        let already_exists = sql::select(sql::exists(
            authors::table.filter(authors::email.eq(body.email.as_str())),
        ))
        .get_result(conn)?;
        if already_exists {
            return Ok(utils::response::error(
                StatusCode::Forbidden,
                "an author already exists for this email.",
            ));
        }

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
                hashed_passwd.as_ref(),
                &mut out,
            );
            hex::encode(out.as_ref())
        };

        //? Insert the new author data.
        let new_author = NewAuthor {
            email: body.email.as_str(),
            name: body.name.as_str(),
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
        let token = utils::auth::generate_token();
        let (token, _) = token.split_at(25);

        //? Store the new registry token in the database.
        let new_author_token = NewAuthorToken {
            name: "API",
            token,
            author_id,
        };
        diesel::insert_into(author_tokens::table)
            .values(new_author_token)
            .execute(conn)?;

        let response = ResponseBody {
            token: String::from(token),
        };
        Ok(utils::response::json(&response))
    });

    transaction.await
}
