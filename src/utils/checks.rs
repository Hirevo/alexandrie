use diesel::dsl as sql;
use diesel::prelude::*;
use http::HeaderMap;

use crate::db::models::Author;
use crate::db::schema::*;
use crate::db::Connection;
use crate::error::Error;

/// Checks if a crate exists in the database given a connection and the crate's name.
pub fn crate_exists(conn: &Connection, name: &str) -> Result<bool, Error> {
    let exists: bool =
        sql::select(sql::exists(crates::table.filter(crates::name.eq(name)))).get_result(conn)?;

    Ok(exists)
}

/// Checks if a user is an author of the named crate.
pub fn is_crate_author(conn: &Connection, crate_name: &str, author_id: i64) -> Result<bool, Error> {
    let exists: bool = sql::select(sql::exists(
        crate_authors::table
            .inner_join(authors::table)
            .inner_join(crates::table)
            .filter(crates::name.eq(crate_name))
            .filter(authors::id.eq(author_id)),
    ))
    .get_result(conn)?;

    Ok(exists)
}

/// Determines the author from the request's headers.
pub fn get_author(conn: &Connection, headers: &HeaderMap) -> Option<Author> {
    let token = headers
        .get("authorization")
        .and_then(|x| x.to_str().ok())?
        .to_owned();

    //? Get the author associated to this token.
    author_tokens::table
        .inner_join(authors::table)
        .select(authors::all_columns)
        .filter(author_tokens::token.eq(token))
        .first::<Author>(conn)
        .ok()
}
