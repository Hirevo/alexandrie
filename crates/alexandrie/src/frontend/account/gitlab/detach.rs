use diesel::prelude::*;
use tide::Request;

use crate::db::schema::authors;
use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(req: Request<State>) -> tide::Result {
    let author = match req.get_author() {
        Some(author) => author,
        None => {
            return Ok(utils::response::redirect("/account/manage"));
        }
    };

    (req.state().db)
        .run(move |conn| {
            diesel::update(authors::table.find(author.id))
                .set(authors::gitlab_id.eq(None::<String>))
                .execute(conn)
        })
        .await?;

    Ok(utils::response::redirect("/account/manage"))
}
