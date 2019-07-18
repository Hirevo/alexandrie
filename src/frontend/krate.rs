use std::sync::Arc;

use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::prelude::*;
use json::json;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::db::DbConn;
use crate::error::{AlexError, Error};
use crate::frontend::config::Config;
use crate::frontend::helpers;

#[get("/crates/<name>")]
pub(crate) fn route(
    config: State<Arc<Config>>,
    conn: DbConn,
    name: String,
) -> Result<Template, Error> {
    let crate_desc: CrateRegistration = crates::table
        .filter(crates::name.eq(name.as_str()))
        .first(&conn.0)
        .optional()?
        .ok_or_else(|| Error::from(AlexError::CrateNotFound(name)))?;
    Ok(Template::render(
        "index",
        json!({
            "instance": config.as_ref(),
            "crate_desc": {
                "id": crate_desc.id,
                "name": crate_desc.name,
                "description": crate_desc.description,
                "downloads": helpers::humanize_number(crate_desc.downloads),
                "created_at": helpers::humanize_datetime(crate_desc.created_at),
                "updated_at": helpers::humanize_datetime(crate_desc.updated_at),
                "documentation": crate_desc.documentation,
                "repository": crate_desc.repository,
            },
        }),
    ))
}
