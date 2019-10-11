use std::num::NonZeroU32;

use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};
use tide::querystring::ContextExt;
use tide::{Context, Response};

use crate::db::models::CrateRegistration;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::Error;
use crate::frontend::helpers;
use crate::index::Indexer;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SearchParams {
    pub q: String,
    pub page: Option<NonZeroU32>,
}

pub(crate) async fn get(ctx: Context<State>) -> Result<Response, Error> {
    let params = ctx.url_query::<SearchParams>().unwrap();
    let searched_text = params.q.clone();
    let q = format!("%{0}%", params.q.replace('\\', "\\\\").replace('%', "\\%"));
    let page_number = params.page.map_or_else(|| 1, |page| page.get());

    let user = ctx.get_author();
    let state = ctx.state();
    let repo = &state.repo;

    let transaction = repo.transaction(|conn| {
        //? Get the total count of search results.
        let total_results = crates::table
            .select(sql::count(crates::id))
            .filter(crates::name.like(q.as_str()))
            .first::<i64>(conn)?;

        //? Get the search results for the given page number.
        let results: Vec<CrateRegistration> = crates::table
            .filter(crates::name.like(q.as_str()))
            .limit(15)
            .offset(15 * i64::from(page_number - 1))
            .load(conn)?;

        let results = results
            .into_iter()
            .map(|result| {
                let keywords = crate_keywords::table
                    .inner_join(keywords::table)
                    .select(keywords::name)
                    .filter(crate_keywords::crate_id.eq(result.id))
                    .load::<String>(conn)?;
                Ok((result, keywords))
            })
            .collect::<Result<Vec<(CrateRegistration, Vec<String>)>, Error>>()?;

        //? Make page number starts counting from 1 (instead of 0).
        let page_count = total_results / 15
            + if total_results > 0 && total_results % 15 == 0 {
                0
            } else {
                1
            };

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
            "instance": &state.frontend.config,
            "searched_text": searched_text,
            "page_number": page_number,
            "page_count": page_count,
            "total_results": total_results,
            "results": results.into_iter().map(|(krate, keywords)| {
                let version = state.index.latest_crate(&krate.name)?.vers;
                let created_at =
                    chrono::NaiveDateTime::parse_from_str(krate.created_at.as_str(), DATETIME_FORMAT)
                        .unwrap();
                let updated_at =
                    chrono::NaiveDateTime::parse_from_str(krate.updated_at.as_str(), DATETIME_FORMAT)
                        .unwrap();
                Ok(json!({
                    "id": krate.id,
                    "name": krate.name,
                    "version": version,
                    "description": krate.description,
                    "created_at": helpers::humanize_datetime(created_at),
                    "updated_at": helpers::humanize_datetime(updated_at),
                    "downloads": helpers::humanize_number(krate.downloads),
                    "documentation": krate.documentation,
                    "repository": krate.repository,
                    "keywords": keywords,
                }))
            }).collect::<Result<Vec<_>, Error>>()?,
        });
        Ok(utils::response::html(
            engine.render("search", &context).unwrap(),
        ))
    });

    transaction.await
}
