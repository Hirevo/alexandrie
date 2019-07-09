use std::sync::{Arc, Mutex};

use diesel::prelude::*;
use json::json;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::db::DbConn;
use crate::error::Error;
use crate::frontend::config::Config;
use crate::frontend::helpers;
use crate::index::Indexer;
use crate::state::AppState;

#[get("/search?<q>&<page>")]
pub(crate) fn route(
    state: State<Arc<Mutex<AppState>>>,
    config: State<Config>,
    conn: DbConn,
    q: String,
    page: Option<u32>,
) -> Result<Template, Error> {
    let searched_text = q.clone();
    let q = format!("%{0}%", q.replace('\\', "\\\\").replace('%', "\\%"));
    let page = page.unwrap_or(1);

    let total_results = crates::table
        .select(diesel::dsl::count(crates::id))
        .filter(crates::name.like(q.as_str()))
        .first::<i64>(&conn.0)?;

    let results = crates::table
        .filter(crates::name.like(q.as_str()))
        .limit(15)
        .offset(15 * ((page - 1) as i64))
        .load::<CrateRegistration>(&conn.0)?;

    let state = state.lock().unwrap();
    Ok(Template::render(
        "search",
        json!({
            "instance": config.inner(),
            "searched_text": searched_text,
            "page_number": page,
            "total_results": total_results,
            "results": results.into_iter().map(|krate| {
                dbg!(&krate);
                let version = dbg!(state.index().latest_crate(&krate.name))?.vers;
                Ok(json!({
                    "id": krate.id,
                    "name": krate.name,
                    "version": version,
                    "description": krate.description,
                    "created_at": helpers::humanize_datetime(krate.created_at),
                    "updated_at": helpers::humanize_datetime(krate.updated_at),
                    "downloads": krate.downloads,
                    "documentation": krate.documentation,
                    "repository": krate.repository,
                }))
            }).collect::<Result<Vec<_>, Error>>()?,
        }),
    ))
}
