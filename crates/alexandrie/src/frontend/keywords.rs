use std::num::NonZeroU32;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use diesel::dsl as sql;
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
    pub page: Option<NonZeroU32>,
}

fn paginated_url(keyword: &str, page_number: u32, page_count: u32) -> Option<String> {
    if page_number >= 1 && page_number <= page_count {
        Some(format!("/keywords/{0}?page={1}", keyword, page_number))
    } else {
        None
    }
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
    Path(keyword): Path<String>,
    user: Option<Auth>,
) -> Result<Either<Html<String>, Redirect>, FrontendError> {
    let page_number = params.page.map_or_else(|| 1, |page| page.get());

    if state.is_login_required() && user.is_none() {
        return Ok(Either::E2(Redirect::to("/account/login")));
    }

    let db = &state.db;
    let state = Arc::clone(&state);

    let transaction = db.transaction(move |conn| {

        //? Get the total count of search results.
        let total_results = crate_keywords::table
            .inner_join(keywords::table)
            .inner_join(crates::table)
            .filter(keywords::name.eq(&keyword))
            .select(sql::count(crates::id))
            .first::<i64>(conn)?;

        //? Get the search results for the given page number.
        //? First get all ids of crates with given keywords
        let results = crate_keywords::table
            .inner_join(keywords::table)
            .inner_join(crates::table)
            .filter(keywords::name.eq(&keyword))
            .select(crates::id)
            .order_by(crates::downloads.desc())
            .limit(15)
            .offset(15 * i64::from(page_number - 1))
            .load::<i64>(conn)?;

        let results = results
            .into_iter()
            .map(|crate_ids| {
                let keywords = crate_keywords::table
                    .inner_join(keywords::table)
                    .select(keywords::name)
                    .filter(crate_keywords::crate_id.eq(crate_ids))
                    .load::<String>(conn)?;

                let keyword_crate: Crate = crates::table
                    .filter(crates::id.eq(crate_ids))
                    .first(conn)?;

                Ok((keyword_crate, keywords))
            })
            .collect::<Result<Vec<(Crate, Vec<String>)>, Error>>()?;

        //? Make page number starts counting from 1 (instead of 0).
        let page_count = (total_results / 15
            + if total_results > 0 && total_results % 15 == 0 {
                0
            } else {
                1
            }) as u32;

        let next_page = paginated_url(&keyword, page_number + 1, page_count);
        let prev_page = paginated_url(&keyword, page_number - 1, page_count);

        let auth = &state.frontend.config.auth;
        let engine = &state.frontend.handlebars;
        let context = json!({
            "auth_disabled": !auth.enabled(),
            "registration_disabled": !auth.allow_registration(),
            "name": keyword,
            "user": user.map(|it| it.into_inner()),
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
        Ok(Either::E1(Html(
            engine.render("keywords", &context).unwrap(),
        )))
    });

    transaction.await
}
