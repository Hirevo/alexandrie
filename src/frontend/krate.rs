use std::sync::{Arc, Mutex};

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
use crate::state::AppState;
use crate::index::Indexer;
use crate::storage::Store;

#[get("/crates/<name>")]
pub(crate) fn route(
    config: State<Arc<Config>>,
    state: State<Arc<Mutex<AppState>>>,
    conn: DbConn,
    name: String,
) -> Result<Template, Error> {
    let state = state.lock().unwrap();
    let crate_desc: CrateRegistration = crates::table
        .filter(crates::name.eq(&name))
        .first(&conn.0)
        .optional()?
        .ok_or_else(|| Error::from(AlexError::CrateNotFound(name)))?;
    let krate = state.index().latest_crate(&crate_desc.name)?;
    let rendered_readme = state.storage().get_readme(&crate_desc.name, krate.vers.clone()).ok();
    Ok(Template::render(
        "crate",
        json!({
            "instance": config.as_ref(),
            "crate": {
                "id": crate_desc.id,
                "name": crate_desc.name,
                "version": krate.vers,
                "description": crate_desc.description,
                "downloads": helpers::humanize_number(crate_desc.downloads),
                "created_at": helpers::humanize_datetime(crate_desc.created_at),
                "updated_at": helpers::humanize_datetime(crate_desc.updated_at),
                "documentation": crate_desc.documentation,
                "repository": crate_desc.repository,
            },
            "rendered_readme": rendered_readme,
        }),
    ))
}
