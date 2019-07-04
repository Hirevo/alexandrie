use diesel::prelude::*;
use diesel::result::Error as SQLError;
use rocket::http::Status;
use rocket::request::{self, FromRequest, Request};
use rocket::Outcome;
use serde::{Deserialize, Serialize};

use crate::db::models::Author;
use crate::db::schema::*;
use crate::{AlexError, DbConn, Error};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Auth(Author, String);

impl<'a, 'r> FromRequest<'a, 'r> for Auth {
    type Error = crate::Error;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        if let Some(token) = request.headers().get_one("Authorization") {
            let conn: DbConn = request
                .guard()
                .map_failure(|(status, _)| (status, Error::from(AlexError::InvalidToken)))?;

            let queried = author_tokens::table
                .inner_join(authors::table)
                .select(authors::all_columns)
                .filter(author_tokens::token.eq(token))
                .first::<Author>(&conn.0);

            match queried {
                Ok(author) => Outcome::Success(Auth(author, String::from(token))),
                Err(SQLError::NotFound) => Outcome::Failure((
                    Status::new(401, "invalid token"),
                    Error::from(AlexError::InvalidToken),
                )),
                Err(err) => {
                    Outcome::Failure((Status::new(500, "internal server error"), Error::from(err)))
                }
            }
        } else {
            Outcome::Failure((
                Status::new(401, "missing token"),
                Error::from(AlexError::InvalidToken),
            ))
        }
    }
}
