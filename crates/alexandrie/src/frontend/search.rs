use std::num::NonZeroUsize;
use std::sync::Arc;

use axum::extract::Query;
use axum::extract::State;
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};

use alexandrie_index::Indexer;

use crate::config::AppState;
use crate::db::models::Crate;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::{Error, FrontendError};
use crate::frontend::helpers;
use crate::utils::auth::frontend::Auth;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryParams {
    pub q: String,
    pub page: Option<NonZeroUsize>,
}

/// Route to search through crates (used by `cargo search`) using tantivy index
pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
    user: Option<Auth>,
) -> Result<Either<Html<String>, Redirect>, FrontendError> {
    let searched_text = params.q.clone();
    let page_number = params.page.map_or_else(|| 1, |page| page.get());

    let offset = (page_number - 1) * crate::fts::DEFAULT_RESULT_PER_PAGE;

    if state.is_login_required() && user.is_none() {
        return Ok(Either::E2(Redirect::to("/account/login")));
    }

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

    let repo = &state.db;
    let state = Arc::clone(&state);

    let transaction = repo.transaction(move |conn| {
        let results: Vec<(Crate, Vec<String>)> = results
            .into_iter()
            .map(|v| {
                let krate = crates::table
                    .filter(crates::id.eq(v))
                    .first::<Crate>(conn)?;
                let keywords = crate_keywords::table
                    .inner_join(keywords::table)
                    .select(keywords::name)
                    .filter(crate_keywords::crate_id.eq(krate.id))
                    .load::<String>(conn)?;
                Ok((krate, keywords))
            })
            .collect::<Result<_, Error>>()?;

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
            "user": user.map(|it| it.into_inner()),
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

        let rendered = engine.render("search", &context)?;
        Ok(Either::E1(Html(rendered)))
    });

    transaction.await
}
