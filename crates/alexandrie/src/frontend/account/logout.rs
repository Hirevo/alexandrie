use tide::Request;

use crate::utils;
use crate::utils::auth::AuthExt;
use crate::State;

pub(crate) async fn get(mut req: Request<State>) -> tide::Result {
    if !req.is_authenticated() {
        return Ok(utils::response::redirect("/"));
    }

    req.session_mut().remove("author.id");

    Ok(utils::response::redirect("/"))
}
