use std::num::NonZeroUsize;

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
    pub page: Option<NonZeroUsize>,
}

/// Route to search through crates (used by `cargo search`) using tantivy index
pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let params = req.query::<SearchParams>().unwrap();
    let searched_text = params.q.clone();
    let page_number = params.page.map_or_else(|| 1, |page| page.get());

    let offset = (page_number - 1) * crate::fts::DEFAULT_RESULT_PER_PAGE;

    let user = req.get_author();
    if req.state().is_login_required() && user.is_none() {
        return Ok(utils::response::redirect("/account/login"));
    }

    let state = req.state().clone();
    let repo = &state.db;

    let (count, results) = state.search.search(
        searched_text.clone(),
        offset,
        crate::fts::DEFAULT_RESULT_PER_PAGE,
    )?;

    let page_count = count / crate::fts::DEFAULT_RESULT_PER_PAGE
        + if count > 0 && count % crate::fts::DEFAULT_RESULT_PER_PAGE == 0 {
            0
        } else {
            1
        };

    let transaction = repo.transaction(move |conn| {
        let state = req.state();

        let results: Vec<(Crate, Vec<String>)> = results
            .into_iter()
            .map(|v| {
                let krate = crates::table
                    .filter(crates::id.eq(v))
                    .first::<Crate>(conn)
                    .unwrap();
                let keywords = crate_keywords::table
                    .inner_join(keywords::table)
                    .select(keywords::name)
                    .filter(crate_keywords::crate_id.eq(krate.id))
                    .load::<String>(conn)
                    .unwrap();
                (krate, keywords)
            })
            .collect();

        let encoded_q = percent_encoding::percent_encode(
            params.q.as_bytes(),
            percent_encoding::NON_ALPHANUMERIC,
        );
        let next_page = if page_number < page_count {
            Some(format!(
                "/search?q={0}&page={1}",
                encoded_q,
                page_number + 1
            ))
        } else {
            None
        };
        let prev_page = if page_number > 1 {
            Some(format!(
                "/search?q={0}&page={1}",
                encoded_q,
                page_number - 1
            ))
        } else {
            None
        };

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
            "instance": &state.frontend.config,
            "searched_text": searched_text,
            "total_results": count,
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
