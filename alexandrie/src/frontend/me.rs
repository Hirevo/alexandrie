use tide::Request;

use crate::utils;
use crate::State;

pub(crate) async fn get(_: Request<State>) -> tide::Result {
    Ok(utils::response::redirect("/account/manage"))
}
