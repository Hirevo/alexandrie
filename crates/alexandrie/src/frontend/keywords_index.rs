use std::sync::Arc;

use axum::extract::State;
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use diesel::dsl as sql;
use diesel::prelude::*;

use json::json;

use crate::config::AppState;
use crate::db::models::Crate;
use crate::db::schema::*;
use crate::error::{Error, FrontendError};
use crate::utils::auth::frontend::Auth;

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    user: Option<Auth>,
) -> Result<Either<Html<String>, Redirect>, FrontendError> {
    if state.is_login_required() && user.is_none() {
        return Ok(Either::E2(Redirect::to("/account/login")));
    }

    let db = &state.db;
    let state = Arc::clone(&state);

    let transaction = db.transaction(move |conn| {
        //? Get the total count of search results.
        let total_results = keywords::table
            .select(sql::count(keywords::id))
            .first::<i64>(conn)?;

        let keywords = keywords::table
            .select(keywords::name)
            .load::<String>(conn)?;

        let auth = &state.frontend.config.auth;
        let engine = &state.frontend.handlebars;
        let context = json!({
            "auth_disabled": !auth.enabled(),
            "registration_disabled": !auth.allow_registration(),
            "user": user.map(|it| it.into_inner()),
            "instance": &state.frontend.config,
            "total_results": total_results,
            "keywords": keywords.into_iter().map(|keyword| {
                let crates = crate_keywords::table
                .inner_join(keywords::table)
                .inner_join(crates::table)
                .filter(keywords::name.eq(&keyword))
                .order_by(crates::downloads.desc())
                .select(crates::id)
                .limit(10)
                .load::<i64>(conn)?
                .into_iter()
                .map(|crate_ids| {
                    let keyword_crate: Crate = crates::table
                        .filter(crates::id.eq(crate_ids))
                        .first(conn)?;
                    Ok(keyword_crate)
                })
                .collect::<Result<Vec<_>, Error>>()?;
            let count = crate_keywords::table
                .inner_join(keywords::table)
                .inner_join(crates::table)
                .filter(keywords::name.eq(&keyword))
                .select(sql::count(crates::id))
                .first::<i64>(conn)?;
            Ok(json!({
                "name":&keyword,
                "crates": crates,
                "count": count
            }))}).collect::<Result<Vec<_>, Error>>()?,
        });
        Ok(Either::E1(Html(
            engine.render("keywords_index", &context).unwrap(),
        )))
    });

    transaction.await
}
