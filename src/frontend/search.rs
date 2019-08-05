use std::num::NonZeroU32;

use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};
use tide::querystring::ContextExt;
use tide::{Context, Response};

use crate::config::State;
use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::error::Error;
use crate::frontend::helpers;
use crate::index::Indexer;
use crate::utils;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchParams {
    q: String,
    page: Option<NonZeroU32>,
}

pub(crate) async fn route(ctx: Context<State>) -> Result<Response, Error> {
    let params = ctx.url_query::<SearchParams>().unwrap();
    let searched_text = params.q.clone();
    let q = format!("%{0}%", params.q.replace('\\', "\\\\").replace('%', "\\%"));
    let page_number = params.page.map_or_else(|| 1, |page| page.get());

    let state = ctx.state();
    let repo = &state.repo;

    let total_results = repo
        .run(|conn| {
            crates::table
                .select(diesel::dsl::count(crates::id))
                .filter(crates::name.like(q.as_str()))
                .first::<i64>(&conn)
        })
        .await?;

    let results = repo
        .run(|conn| {
            crates::table
                .filter(crates::name.like(q.as_str()))
                .limit(15)
                .offset(15 * i64::from(page_number - 1))
                .load::<CrateRegistration>(&conn)
        })
        .await?;

    let page_count = total_results / 15
        + if total_results > 0 && total_results % 15 == 0 {
            0
        } else {
            1
        };

    let engine = &state.frontend.handlebars;
    let context = json!({
        "instance": &state.frontend.config,
        "searched_text": searched_text,
        "page_number": page_number,
        "page_count": page_count,
        "total_results": total_results,
        "results": results.into_iter().map(|krate| {
            let version = state.index.latest_crate(&krate.name)?.vers;
            Ok(json!({
                "id": krate.id,
                "name": krate.name,
                "version": version,
                "description": krate.description,
                "created_at": helpers::humanize_datetime(krate.created_at),
                "updated_at": helpers::humanize_datetime(krate.updated_at),
                "downloads": helpers::humanize_number(krate.downloads),
                "documentation": krate.documentation,
                "repository": krate.repository,
            }))
        }).collect::<Result<Vec<_>, Error>>()?,
    });
    Ok(utils::response::html(
        engine.render("search", &context).unwrap(),
    ))
}
