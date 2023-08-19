use std::sync::Arc;

use axum::extract::State;
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::dsl as sql;
use diesel::prelude::*;
use json::json;

use crate::config::AppState;
use crate::db::models::Author;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::FrontendError;
use crate::frontend::helpers;

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    user: Option<Author>,
) -> Result<Either<Html<String>, Redirect>, FrontendError> {
    if state.is_login_required() && user.is_none() {
        return Ok(Either::E2(Redirect::to("/account/login")));
    }

    let db = &state.db;
    let state = Arc::clone(&state);
    let transaction = db.transaction(move |conn| {
        //? Get total number of crates.
        let crate_count: i64 = crates::table
            .select(sql::count(crates::id))
            .first(conn)?;

        //? Get total number of crate downloads.
        let total_downloads = crates::table
            .select(sql::sum(crates::downloads))
            .first::<Option<BigDecimal>>(conn)?
            .map_or(0, |dec| {
                dec.to_u64()
                    .expect("download count exceeding u64::max_value()")
            });

        //? Get the 10 most downloaded crates.
        let most_downloaded: Vec<(String, i64)> = crates::table
            .select((crates::name, crates::downloads))
            .order_by(crates::downloads.desc())
            .limit(10)
            .load(conn)?;

        //? Get the 10 most recently updated crates.
        let last_updated: Vec<(String, String)> = crates::table
            .select((crates::name, crates::updated_at))
            .order_by(crates::updated_at.desc())
            .limit(10)
            .load(conn)?;

        let engine = &state.frontend.handlebars;
        let context = json!({
            "user": user,
            "instance": &state.frontend.config,
            "total_downloads": helpers::humanize_number(total_downloads),
            "crate_count": helpers::humanize_number(crate_count),
            "most_downloaded": most_downloaded.into_iter().map(|(name, downloads)| json!({
                "name": name,
                "downloads": helpers::humanize_number(downloads),
            })).collect::<Vec<_>>(),
            "last_updated": last_updated.into_iter().map(|(name, date)| {
                let updated_at = chrono::NaiveDateTime::parse_from_str(date.as_str(), DATETIME_FORMAT).unwrap();
                json!({
                    "name": name,
                    "updated_at": helpers::humanize_datetime(updated_at),
                })
            }).collect::<Vec<_>>(),
        });

        let rendered = engine.render("index", &context)?;

        Ok::<_, FrontendError>(Either::E1(Html(rendered)))
    });

    transaction.await
}
