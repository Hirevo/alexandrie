use diesel::prelude::*;
use http::status::StatusCode;
use json::json;
use serde::{Deserialize, Serialize};
use tide::{Request, Response};

use crate::db::models::{Author, NewCrateAuthor};
use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::utils;
use crate::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OwnerListResponse {
    pub users: Vec<OwnerListEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OwnerListEntry {
    pub id: i64,
    pub login: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OwnerAddBody {
    /// Owners' emails to add.
    pub users: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct OwnerDeleteBody {
    /// Owners' emails to delete.
    pub users: Vec<String>,
}

pub(crate) async fn get(req: Request<State>) -> Result<Response, Error> {
    let name = req.param::<String>("name").unwrap();

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        //? Does this crate exists?
        let exists = utils::checks::crate_exists(conn, name.as_str())?;
        if !exists {
            return Ok(utils::response::error(
                StatusCode::NOT_FOUND,
                format!("no crates named '{0}' could be found", name),
            ));
        }

        //? Get all authors of this crate.
        let authors = crate_authors::table
            .inner_join(authors::table)
            .inner_join(crates::table)
            .select(authors::all_columns)
            .filter(crates::name.eq(name.as_str()))
            .load::<Author>(conn)?;

        let users = authors
            .into_iter()
            .map(|author| {
                let Author {
                    id,
                    email: login,
                    name,
                    ..
                } = author;
                OwnerListEntry { id, login, name }
            })
            .collect();

        let data = OwnerListResponse { users };
        Ok(utils::response::json(&data))
    });

    transaction.await
}

pub(crate) async fn put(mut req: Request<State>) -> Result<Response, Error> {
    let name = req.param::<String>("name").unwrap();
    let OwnerAddBody { users: new_authors } = req.body_json().await?;

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        let author =
            utils::checks::get_author(conn, req.headers()).ok_or(AlexError::InvalidToken)?;

        //? Get this crate's ID.
        let crate_id = crates::table
            .select(crates::id)
            .filter(crates::name.eq(name.as_str()))
            .first::<i64>(conn)
            .optional()?;
        let crate_id = match crate_id {
            Some(id) => id,
            None => {
                return Ok(utils::response::error(
                    StatusCode::NOT_FOUND,
                    format!("no crates named '{0}' could be found", name),
                ))
            }
        };

        //? Get all authors of this crate.
        let crate_authors = crate_authors::table
            .inner_join(authors::table)
            .inner_join(crates::table)
            .select(authors::id)
            .filter(crates::name.eq(name.as_str()))
            .load::<i64>(conn)?;

        //? Check if user is one of these authors.
        if !crate_authors.contains(&author.id) {
            return Ok(utils::response::error(
                StatusCode::FORBIDDEN,
                "you are not an author of this crate",
            ));
        }

        //? Get all registered authors which:
        //?   - are not authors of this crate.
        //?   - are one of the requested new authors.
        let new_authors = authors::table
            .select((authors::id, authors::name))
            .filter(authors::id.ne_all(crate_authors.as_slice()))
            .filter(authors::email.eq_any(new_authors.as_slice()))
            .load::<(i64, String)>(conn)?;

        let (new_authors_ids, new_authors_names) = {
            let len = new_authors.len();
            let (mut fsts, mut snds) = (Vec::with_capacity(len), Vec::with_capacity(len));
            for (fst, snd) in new_authors.into_iter() {
                fsts.push(fst);
                snds.push(snd);
            }
            (fsts, snds)
        };

        //? Prepare requests structures.
        let new_authors = new_authors_ids
            .into_iter()
            .map(|author_id| NewCrateAuthor {
                crate_id,
                author_id,
            })
            .collect::<Vec<_>>();

        //? Insert the new authors.
        diesel::insert_into(crate_authors::table)
            .values(new_authors)
            .execute(&**conn)?;

        let authors_list = match new_authors_names.len() {
            0 => String::new(),
            1 => new_authors_names.into_iter().next().unwrap(),
            2 => new_authors_names.join(" and "),
            _ => {
                let fsts = new_authors_names[..new_authors_names.len() - 1].join(", ");
                let last = new_authors_names.into_iter().last().unwrap();
                [fsts, last].join(" and ")
            }
        };

        let data = json!({
            "ok": "true",
            "msg": format!("{0} has been added as authors of {1}", authors_list, name),
        });
        Ok(utils::response::json(&data))
    });

    transaction.await
}

pub(crate) async fn delete(mut req: Request<State>) -> Result<Response, Error> {
    let name = req.param::<String>("name").unwrap();
    let OwnerDeleteBody { users: old_authors } = req.body_json().await?;

    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        let author =
            utils::checks::get_author(conn, req.headers()).ok_or(AlexError::InvalidToken)?;

        //? Get this crate's ID.
        let crate_id = crates::table
            .select(crates::id)
            .filter(crates::name.eq(name.as_str()))
            .first::<i64>(conn)
            .optional()?;
        let crate_id = match crate_id {
            Some(id) => id,
            None => {
                return Ok(utils::response::error(
                    StatusCode::NOT_FOUND,
                    format!("no crates named '{0}' could be found", name),
                ))
            }
        };

        //? Get all authors of this crate.
        let crate_authors = crate_authors::table
            .inner_join(authors::table)
            .inner_join(crates::table)
            .select(authors::id)
            .filter(crates::name.eq(name.as_str()))
            .load::<i64>(conn)?;

        //? Check if user is one of these authors.
        if !crate_authors.contains(&author.id) {
            return Ok(utils::response::error(
                StatusCode::FORBIDDEN,
                "you are not an author of this crate",
            ));
        }

        //? Get all registered authors which:
        //?   - are authors of this crate.
        //?   - are one of the requested new authors.
        let old_authors = authors::table
            .select((authors::id, authors::name))
            .filter(authors::id.eq_any(crate_authors.as_slice()))
            .filter(authors::email.eq_any(old_authors.as_slice()))
            .load::<(i64, String)>(conn)?;

        //? Check if there will remain at least one author for this crate.
        if crate_authors.len() == 1 && old_authors.len() == 1 {
            return Ok(utils::response::error(
                StatusCode::BAD_REQUEST,
                "cannot leave the crate without any authors",
            ));
        }

        //? Split IDs and names into separate vectors.
        let (old_authors_ids, old_authors_names) = {
            let len = old_authors.len();
            let (mut fsts, mut snds) = (Vec::with_capacity(len), Vec::with_capacity(len));
            for (fst, snd) in old_authors.into_iter() {
                fsts.push(fst);
                snds.push(snd);
            }
            (fsts, snds)
        };

        //? Delete from authors.
        diesel::delete(
            crate_authors::table
                .filter(crate_authors::crate_id.eq(crate_id))
                .filter(crate_authors::author_id.eq_any(old_authors_ids.as_slice())),
        )
        .execute(conn)?;

        let authors_list = match old_authors_names.len() {
            0 => String::new(),
            1 => old_authors_names.into_iter().next().unwrap(),
            2 => old_authors_names.join(" and "),
            _ => {
                let fsts = old_authors_names[..old_authors_names.len() - 1].join(", ");
                let last = old_authors_names.into_iter().last().unwrap();
                [fsts, last].join(" and ")
            }
        };

        let data = json!({
            "ok": "true",
            "msg": format!("{0} has been removed from authors of {1}", authors_list, name),
        });
        Ok(utils::response::json(&data))
    });

    transaction.await
}
