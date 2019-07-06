use diesel::prelude::*;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use serde::{Deserialize, Serialize};

use crate::db::models::Author;
use crate::db::schema::*;
use crate::{AlexError, DbConn, Error};

/// The Auth request guard.  
/// It checks that the incoming request is from an authenticated user.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Auth(Author, String);

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = crate::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        if let Some(token) = request.headers().get_one("Authorization") {
            let conn: DbConn = request
                .guard()
                .map_failure(|(status, _)| (status, Error::from(AlexError::InvalidToken)))?;

            author_tokens::table
                .inner_join(authors::table)
                .select(authors::all_columns)
                .filter(author_tokens::token.eq(token))
                .first::<Author>(&conn.0)
                .optional()
                .map_err(|err| Err((Status::new(500, "internal server error"), Error::from(err))))?
                .map_or_else(
                    || {
                        Outcome::Failure((
                            Status::new(401, "invalid token"),
                            Error::from(AlexError::InvalidToken),
                        ))
                    },
                    |author| Outcome::Success(Auth(author, String::from(token))),
                )
        } else {
            Outcome::Failure((
                Status::new(401, "missing token"),
                Error::from(AlexError::InvalidToken),
            ))
        }
    }
}
