use diesel::prelude::*;
use tide::cookies::ContextExt as CookieExt;
use tide::{Context, Response};

use crate::db::schema::*;
use crate::error::Error;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(mut ctx: Context<State>) -> Result<Response, Error> {
    if !ctx.is_authenticated() {
        return Ok(utils::response::redirect("/"));
    }

    let cookie = ctx.get_cookie(utils::auth::COOKIE_NAME).unwrap().unwrap();

    let state = ctx.state();
    let repo = &state.repo;

    repo.run(|conn| {
        //? Delete the session.
        diesel::delete(sessions::table.filter(sessions::token.eq(cookie.value()))).execute(conn)
    })
    .await?;

    ctx.remove_cookie(cookie).unwrap();

    Ok(utils::response::redirect("/"))
}
