use diesel::prelude::*;
use json::json;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::db::DbConn;
use crate::error::Error;
use crate::frontend::config::Config;

#[get("/search?<q>&<page>")]
pub(crate) fn route(
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

    Ok(Template::render(
        "search",
        json!({
            "instance": config.inner(),
            "searched_text": searched_text,
            "page_number": page,
            "total_results": total_results,
            "results": results,
        }),
    ))
}
