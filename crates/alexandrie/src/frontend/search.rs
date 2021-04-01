use std::num::NonZeroU32;

use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};
use tide::Request;

use alexandrie_index::Indexer;

use crate::db::models::Crate;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::Error;
use crate::frontend::helpers;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct SearchParams {
    pub q: String,
    pub page: Option<NonZeroU32>,
}

pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let params = req.query::<SearchParams>()?;
    let searched_text = utils::canonical_name(params.q.as_str());
    let q = format!(
        "%{0}%",
        searched_text.replace('\\', "\\\\").replace('%', "\\%")
    );

    let page_number = params.page.map_or_else(|| 1, |page| page.get());

    let user = req.get_author();
    let state = req.state().clone();
    let repo = &state.repo;

    let transaction = repo.transaction(move |conn| {
        let state = req.state();

        //? Get the total count of search results.
        let total_results = crates::table
            .select(sql::count(crates::id))
            .filter(crates::canon_name.like(q.as_str()))
            .first::<i64>(conn)?;

        //? Get the search results for the given page number.
        let results: Vec<Crate> = crates::table
            .filter(crates::canon_name.like(q.as_str()))
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
            .collect::<Result<Vec<(Crate, Vec<String>)>, Error>>()?;

        //? Make page number starts counting from 1 (instead of 0).
        let page_count = (total_results / 15
            + if total_results > 0 && total_results % 15 == 0 {
                0
            } else {
                1
            }) as u32;

        let encoded_q = percent_encoding::percent_encode(
            params.q.as_bytes(),
            percent_encoding::NON_ALPHANUMERIC,
        );
        let next_page = if page_number < page_count {
            Some(format!("/search?q={0}&page={1}", encoded_q, page_number + 1))
        } else {
            None
        };
        let prev_page = if page_number > 1 {
            Some(format!("/search?q={0}&page={1}", encoded_q, page_number - 1))
        } else {
            None
        };

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
            "instance": &state.frontend.config,
            "searched_text": params.q,
            "total_results": total_results,
            "pagination": {
                "current": page_number,
                "total_count": page_count,
                "next": next_page,
                "prev": prev_page,
            },
            "results": results.into_iter().map(|(krate, keywords)| {
                let record = state.index.latest_record(&krate.name)?;
                let created_at =
                    chrono::NaiveDateTime::parse_from_str(krate.created_at.as_str(), DATETIME_FORMAT)
                        .unwrap();
                let updated_at =
                    chrono::NaiveDateTime::parse_from_str(krate.updated_at.as_str(), DATETIME_FORMAT)
                        .unwrap();
                Ok(json!({
                    "id": krate.id,
                    "name": krate.name,
                    "version": record.vers,
                    "description": krate.description,
                    "created_at": helpers::humanize_datetime(created_at),
                    "updated_at": helpers::humanize_datetime(updated_at),
                    "downloads": helpers::humanize_number(krate.downloads),
                    "documentation": krate.documentation,
                    "repository": krate.repository,
                    "keywords": keywords,
                    "yanked": record.yanked,
                }))
            }).collect::<Result<Vec<_>, Error>>()?,
        });
        Ok(utils::response::html(
            engine.render("search", &context)?,
        ))
    });

    transaction.await
}
