#![feature(async_await, async_closure)]
#![allow(clippy::redundant_closure, clippy::needless_lifetimes, unused)]

#[macro_use]
extern crate diesel;

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use diesel::MysqlConnection;
use path_absolutize::Absolutize;
use tide::error::ResultExt;
use tide::middleware::RequestLogger;
use tide::{App, Context, Response};

pub mod api;
pub mod config;
pub mod db;
pub mod error;
pub mod index;
pub mod krate;
pub mod logs;
pub mod storage;
pub mod utils;

#[cfg(feature = "frontend")]
pub mod frontend;

use crate::config::{Config, State};
use crate::error::Error;

pub type Repo = db::Repo<MysqlConnection>;

#[cfg(feature = "frontend")]
async fn assets(ctx: Context<State>) -> tide::EndpointResult {
    let path = ctx.param::<PathBuf>("path").unwrap();
    let served = Path::new("assets").absolutize().server_err()?;
    let path = served.join(path).absolutize().client_err()?;

    if path.starts_with(served) {
        let file = fs::read(path).server_err()?;
        Ok(tide::http::Response::builder()
            .status(tide::http::StatusCode::OK)
            .header("content-length", file.len().to_string())
            .body(tide::Body::from(file))
            .unwrap())
    } else {
        Err(tide::error::Error::from(tide::http::StatusCode::NOT_FOUND))
    }
}

#[runtime::main(runtime_tokio::Tokio)]
async fn main() -> io::Result<()> {
    logs::init();

    let contents = fs::read("alexandrie.toml")?;
    let config: Config = toml::from_slice(contents.as_slice()).expect("invalid configuration");

    #[cfg(feature = "frontend")]
    let frontend_enabled = config.frontend.enabled;

    let state: State = config.into();
    let mut app: App<State> = App::with_state(state);

    app.middleware(RequestLogger::new());

    app.at("/api/v1/crates/new").put(api::publish::route);
    app.at("/api/v1/crates").get(api::search::route);
    app.at("/api/v1/crates/:name/:version/download")
        .get(api::download::route);

    #[cfg(feature = "frontend")]
    {
        if frontend_enabled {
            app.at("/").get(frontend::index::route);
            app.at("/search").get(frontend::search::route);
            app.at("/crates/:crate").get(frontend::krate::route);

            app.at("/assets/*path").get(assets);
        }
    }

    app.serve("127.0.0.1:3000").await?;

    Ok(())
}
