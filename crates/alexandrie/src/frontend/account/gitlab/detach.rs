use diesel::prelude::*;
use tide::{Request, StatusCode};

use crate::db::schema::authors;
use crate::frontend::account::utils::count_auth_methods;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let Some(author) = req.get_author() else {
        return Ok(utils::response::redirect("/account/manage"));
    };

    if count_auth_methods(&author) < 2 {
        return utils::response::error_html(
            req.state(),
            Some(author),
            StatusCode::BadRequest,
            "too few authentication methods remaining",
        );
    }

    (req.state().db)
        .run(move |conn| {
            diesel::update(authors::table.find(author.id))
                .set(authors::gitlab_id.eq(None::<String>))
                .execute(conn)
        })
        .await?;

    Ok(utils::response::redirect("/account/manage"))
}
