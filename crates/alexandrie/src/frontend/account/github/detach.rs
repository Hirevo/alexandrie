use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Redirect;
use axum_extra::either::Either;
use axum_extra::response::Html;
use diesel::prelude::*;

use crate::config::AppState;
use crate::db::models::Author;
use crate::db::schema::authors;
use crate::error::FrontendError;
use crate::frontend::account::utils::count_auth_methods;
use crate::utils;

pub(crate) async fn get(
    State(state): State<Arc<AppState>>,
    maybe_author: Option<Author>,
) -> Result<Either<(StatusCode, Html<String>), Redirect>, FrontendError> {
    let Some(author) = maybe_author else {
        return Ok(Either::E2(Redirect::to("/account/manage")));
    };

    if count_auth_methods(&author) < 2 {
        let rendered = utils::response::error_html(
            state.as_ref(),
            Some(author),
            "too few authentication methods remaining",
        )?;
        return Ok(Either::E1((StatusCode::BAD_REQUEST, Html(rendered))));
    }

    let db = &state.db;
    db.run(move |conn| {
        diesel::update(authors::table.find(author.id))
            .set(authors::github_id.eq(None::<String>))
            .execute(conn)
    })
    .await?;

    Ok(Either::E2(Redirect::to("/account/manage")))
}
