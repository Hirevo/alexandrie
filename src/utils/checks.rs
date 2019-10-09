use diesel::connection::Connection;
use diesel::dsl as sql;
use diesel::mysql::Mysql;
use diesel::prelude::*;

use crate::db::schema::*;
use crate::error::Error;

/// Checks if a crate exists in the database given a connection and the crate's name.
pub fn crate_exists<Conn>(conn: &Conn, name: &str) -> Result<bool, Error>
where
    Conn: Connection<Backend = Mysql>,
{
    let exists: bool =
        sql::select(sql::exists(crates::table.filter(crates::name.eq(name)))).get_result(conn)?;

    Ok(exists)
}

/// Checks if a user is an author of the named crate.
pub fn is_crate_author<Conn>(conn: &Conn, crate_name: &str, author_id: u64) -> Result<bool, Error>
where
    Conn: Connection<Backend = Mysql>,
{
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
