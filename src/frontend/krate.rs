use diesel::prelude::*;
use json::json;
use tide::{Context, Response};

use crate::config::State;
use crate::db::models::{CrateKeyword, CrateRegistration, Keyword};
use crate::db::schema::*;
use crate::error::{AlexError, Error};
use crate::frontend::helpers;
use crate::index::Indexer;
use crate::storage::Store;
use crate::utils;

pub(crate) async fn route(ctx: Context<State>) -> Result<Response, Error> {
    let name = ctx.param::<String>("crate").unwrap();
    let state = ctx.state();
    let repo = &state.repo;

    let crate_desc = repo
        .run(|conn| {
            crates::table
                .filter(crates::name.eq(&name))
                .first::<CrateRegistration>(&conn)
                .optional()
        })
        .await?
        .ok_or_else(|| Error::from(AlexError::CrateNotFound(name)))?;
    let krate = state.index.latest_crate(&crate_desc.name)?;
    let rendered_readme = state
        .storage
        .get_readme(&crate_desc.name, krate.vers.clone())
        .ok();
    let keywords = repo
        .run(|conn| {
            CrateKeyword::belonging_to(&crate_desc)
                .inner_join(keywords::table)
                .select(keywords::all_columns)
                .load::<Keyword>(&conn)
        })
        .await?;

    let engine = &state.frontend.handlebars;
    let context = json!({
        "instance": &state.frontend.config,
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
        "keywords": keywords,
    });
    Ok(utils::response::html(
        engine.render("crate", &context).unwrap(),
    ))
}
