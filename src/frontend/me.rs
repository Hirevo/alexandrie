use tide::{Request, Response};

use crate::error::Error;
use crate::utils;
use crate::State;

pub(crate) async fn get(_: Request<State>) -> Result<Response, Error> {
    Ok(utils::response::redirect("/account/manage"))
}
