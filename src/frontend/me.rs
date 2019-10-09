use tide::{Context, Response};

use crate::utils;
use crate::State;

pub(crate) async fn get(_: Context<State>) -> Response {
    utils::response::redirect("/account/manage")
}
