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

use crate::config::AppState;
use crate::db::models::Keyword;
use crate::db::schema::*;
use crate::error::{Error, FrontendError};
use crate::utils::auth::frontend::Auth;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct QueryParams {
    pub q: String,
    pub page: Option<NonZeroU32>,
}

fn paginated_url(query: &str, page_number: u32, page_count: u32) -> Option<String> {
    let query = query.as_bytes();
    let encoded_q = percent_encoding::percent_encode(query, percent_encoding::NON_ALPHANUMERIC);
    if page_number >= 1 && page_number <= page_count {
        Some(format!(
            "/keywords_search?q={0}&page={1}",
            encoded_q, page_number
        ))
    } else {
        None
    }
}

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    Query(params): Query<QueryParams>,
    user: Option<Auth>,
) -> Result<Either<Html<String>, Redirect>, FrontendError> {
    let page_number = params.page.map_or_else(|| 1, |page| page.get());
    let searched_text = params.q.clone();

    if state.is_login_required() && user.is_none() {
        return Ok(Either::E2(Redirect::to("/account/login")));
    }

    let db = &state.db;
    let state = Arc::clone(&state);

    let transaction = db.transaction(move |conn| {
        let escaped_like_query = params.q.replace('\\', "\\\\").replace('%', "\\%");
        let escaped_like_query = format!("%{escaped_like_query}%");

        //? Get the total count of search results.
        let total_results = keywords::table
            .select(sql::count(keywords::id))
            .filter(keywords::name.like(escaped_like_query.as_str()))
            .first::<i64>(conn)?;

        //? Get the search results for the given page number.
        let results: Vec<Keyword> = keywords::table
            .filter(keywords::name.like(escaped_like_query.as_str()))
            .limit(15)
            .offset(15 * i64::from(page_number - 1))
            .load(conn)?;

        let results = results
            .into_iter()
            .map(|result| {
                let crates = crate_keywords::table
                    .inner_join(crates::table)
                    .select(crates::name)
                    .filter(crate_keywords::keyword_id.eq(result.id))
                    .limit(5)
                    .load::<String>(conn)?;
                Ok((result.name, crates))
            })
            .collect::<Result<Vec<(String, Vec<String>)>, Error>>()?;

        //? Make page number starts counting from 1 (instead of 0).
        let page_count = (total_results / 15
            + if total_results > 0 && total_results % 15 == 0 {
                0
            } else {
                1
            }) as u32;

        let next_page = paginated_url(&params.q, page_number + 1, page_count);
        let prev_page = paginated_url(&params.q, page_number - 1, page_count);

        let auth = &state.frontend.config.auth;
        let engine = &state.frontend.handlebars;
        let context = json!({
            "auth_disabled": !auth.enabled(),
            "registration_disabled": !auth.allow_registration(),
            "user": user.map(|it| it.into_inner()),
            "instance": &state.frontend.config,
            "searched_text": searched_text,
            "total_results": total_results,
            "pagination": {
                "current": page_number,
                "total_count": page_count,
                "next": next_page,
                "prev": prev_page,
            },
            "results": results.into_iter().map(|(keyword, crates)| {
                Ok(json!({
                    "name": keyword,
                    "crates": crates,
                }))
            }).collect::<Result<Vec<_>, Error>>()?,
        });
        Ok(Either::E1(Html(
            engine.render("keywords_search", &context).unwrap(),
        )))
    });

    transaction.await
}
