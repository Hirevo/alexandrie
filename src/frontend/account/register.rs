use json::json;
use tide::{Context, Response};

use crate::config::State;
use crate::error::Error;
use crate::utils;

pub(crate) async fn get(ctx: Context<State>) -> Result<Response, Error> {
    let state = ctx.state();
    let repo = &state.repo;

    let engine = &state.frontend.handlebars;
    let context = json!({
        "instance": &state.frontend.config,
    });
    Ok(utils::response::html(
        engine.render("account/register", &context).unwrap(),
    ))
}

pub(crate) async fn post(ctx: Context<State>) -> Result<Response, Error> {
    let state = ctx.state();
    let repo = &state.repo;

    let engine = &state.frontend.handlebars;
    let context = json!({
        "instance": &state.frontend.config,
    });
    Ok(utils::response::html(
        engine.render("account/register", &context).unwrap(),
    ))
}
