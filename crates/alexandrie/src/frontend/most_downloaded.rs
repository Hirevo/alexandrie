use std::num::NonZeroU32;
use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;
use serde::{Deserialize, Serialize};

use alexandrie_index::Indexer;

use crate::config::AppState;
use crate::db::models::{Author, Crate};
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::{Error, FrontendError};
use crate::frontend::helpers;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryParams {
    pub page: Option<NonZeroU32>,
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
    user: Option<Author>,
) -> Result<Either<Html<String>, Redirect>, FrontendError> {
    let page_number = params.page.map_or_else(|| 1, |page| page.get());

    if state.is_login_required() && user.is_none() {
        return Ok(Either::E2(Redirect::to("/account/login")));
    }

    let db = &state.db;
    let state = Arc::clone(&state);

    let transaction = db.transaction(move |conn| {
        //? Get the total count of search results.
        let total_results: i64 = crates::table
            .select(sql::count(crates::id))
            .first(conn)?;

        //? Get the search results for the given page number.
        let results: Vec<Crate> = crates::table
            .order_by(crates::downloads.desc())
            .limit(15)
            .offset(15 * i64::from(page_number - 1))
            .load(conn)?;

        let results: Vec<(Crate, Vec<String>)> = results
            .into_iter()
            .map(|result| {
                let keywords = crate_keywords::table
                    .inner_join(keywords::table)
                    .select(keywords::name)
                    .filter(crate_keywords::crate_id.eq(result.id))
                    .load(conn)?;
                Ok((result, keywords))
            })
            .collect::<Result<_, Error>>()?;

        //? Make page number starts counting from 1 (instead of 0).
        let page_count = (total_results / 15
            + if total_results > 0 && total_results % 15 == 0 {
                0
            } else {
                1
            }) as u32;

        let next_page = if page_number < page_count {
            Some(format!("/most-downloaded?page={0}", page_number + 1))
        } else {
            None
        };
        let prev_page = if page_number > 1 {
            Some(format!("/most-downloaded?page={0}", page_number - 1))
        } else {
            None
        };

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
            "instance": &state.frontend.config,
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

        let rendered = engine.render("most-downloaded", &context)?;

        Ok(Either::E1(Html(rendered)))
    });

    transaction.await
}
