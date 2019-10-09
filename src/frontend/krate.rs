use diesel::prelude::*;
use json::json;
use tide::{Context, Response};

use crate::db::models::{CrateAuthor, CrateCategory, CrateKeyword, CrateRegistration, Keyword};
use crate::db::schema::*;
use crate::error::Error;
use crate::frontend::helpers;
use crate::index::Indexer;
use crate::storage::Store;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(ctx: Context<State>) -> Result<Response, Error> {
    let name = ctx.param::<String>("crate").unwrap();

    let user = ctx.get_author();
    let state = ctx.state();
    let repo = &state.repo;

    let transaction = repo.transaction(|conn| {
        //? Get this crate's data.
        let crate_desc = crates::table
            .filter(crates::name.eq(&name))
            .first::<CrateRegistration>(conn)
            .optional()?;
        let crate_desc = match crate_desc {
            Some(crate_desc) => crate_desc,
            None => {
                let response = utils::response::error_html(
                    state.as_ref(),
                    user,
                    http::StatusCode::NOT_FOUND,
                    format!("No crate named '{0}' has been found.", name),
                );
                return Ok(response);
            }
        };
        let krate = state.index.latest_crate(&crate_desc.name)?;

        //? Get the HTML-rendered README page of this crate.
        let rendered_readme = state
            .storage
            .get_readme(&crate_desc.name, krate.vers.clone())
            .ok();

        //? Get the authors' names of this crate.
        let authors = CrateAuthor::belonging_to(&crate_desc)
            .inner_join(authors::table)
            .select(authors::name)
            .load::<String>(conn)?;

        //? Get the keywords for this crate.
        let keywords = CrateKeyword::belonging_to(&crate_desc)
            .inner_join(keywords::table)
            .select(keywords::all_columns)
            .load::<Keyword>(conn)?;

        //? Get the categories of this crate.
        let categories = CrateCategory::belonging_to(&crate_desc)
            .inner_join(categories::table)
            .select(categories::name)
            .load::<String>(conn)?;

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
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
            "authors": authors,
            "rendered_readme": rendered_readme,
            "keywords": keywords,
            "categories": categories,
        });
        Ok(utils::response::html(
            engine.render("crate", &context).unwrap(),
        ))
    });
    transaction.await
}
