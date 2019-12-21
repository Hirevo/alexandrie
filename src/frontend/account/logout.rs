use diesel::prelude::*;
use tide::{Request, Response};

use crate::db::schema::*;
use crate::error::Error;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::utils::cookies::CookiesExt;
use crate::State;

pub(crate) async fn get(mut req: Request<State>) -> Result<Response, Error> {
    if !req.is_authenticated() {
        return Ok(utils::response::redirect("/"));
    }

    let cookie = req.get_cookie(utils::auth::COOKIE_NAME).unwrap();

    let state = req.state();
    let repo = &state.repo;

    let cloned_cookie = cookie.clone();
    repo.run(move |conn| {
        //? Delete the session.
        diesel::delete(sessions::table.filter(sessions::token.eq(cloned_cookie.value())))
            .execute(conn)
    })
    .await?;

    req.remove_cookie(cookie).unwrap();

    Ok(utils::response::redirect("/"))
}
