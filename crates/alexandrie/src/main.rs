#![warn(unused, missing_docs)]
//!
//! This is an alternative crate registry for use with Cargo, written in Rust.
//!
//! This repository implements the Cargo APIs and interacts with a crate index as specified in the [Cargo's Alternative Registries RFC].  
//! This allows to have a private registry to host crates that are specific to what your doing and not suitable for publication on [crates.io] while maintaining the same build experience as with crates from [crates.io].  
//!
//! [crates.io]: https://crates.io
//! [Cargo's Alternative Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification
//!
//! Goals
//! -----
//!
//! - Offer customizable crate storage strategies (local on-disk, S3, Git Repo, etc...).
//! - Offer multiple backing database options (MySQL, PostgreSQL or SQLite).
//! - An optional integrated (server-side rendered) front-end.
//!

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;
#[macro_use(slog_o)]
extern crate slog;

use std::sync::Arc;

#[cfg(feature = "frontend")]
use std::io;

use async_std::fs;

use clap::{App, Arg};
use tide::http::cookies::SameSite;
use tide::http::mime;
use tide::sessions::SessionMiddleware;
use tide::utils::After;
use tide::{Body, Response, Server};

/// API endpoints definitions.
pub mod api;
/// Configuration and internal state type definitions.
pub mod config;
/// Database abstractions module.
pub mod db;
/// Error-related type definitions.
pub mod error;
/// Logs initialisation.
pub mod logs;
/// Various utilities and helpers.
pub mod utils;

/// Frontend endpoints definitions.
#[cfg(feature = "frontend")]
pub mod frontend;

use crate::config::{Config, FrontendConfig};
use crate::error::Error;
use crate::utils::build;
use crate::utils::request_log::RequestLogger;

#[cfg(feature = "frontend")]
use crate::utils::auth::AuthMiddleware;
#[cfg(feature = "frontend")]
use crate::utils::sessions::SqlStore;

/// The application state type used for the web server.
pub type State = Arc<config::State>;

#[cfg(feature = "mysql")]
embed_migrations!("../../migrations/mysql");
#[cfg(feature = "sqlite")]
embed_migrations!("../../migrations/sqlite");
#[cfg(feature = "postgres")]
embed_migrations!("../../migrations/postgres");

#[cfg(feature = "frontend")]
fn frontend_routes(state: State, frontend_config: FrontendConfig) -> io::Result<Server<State>> {
    let mut app = tide::with_state(Arc::clone(&state));

    let store = SqlStore::new(state.db.clone());

    log::info!("setting up session middleware");
    app.with(
        SessionMiddleware::new(store, frontend_config.sessions.secret.as_bytes())
            .with_cookie_name(frontend_config.sessions.cookie_name.as_str())
            .with_same_site_policy(SameSite::Lax),
    );
    log::info!("setting up authentication middleware");
    app.with(AuthMiddleware::new());

    log::info!("mounting '/'");
    app.at("/").get(frontend::index::get);
    log::info!("mounting '/me'");
    app.at("/me").get(frontend::me::get);
    log::info!("mounting '/search'");
    app.at("/search").get(frontend::search::get);
    log::info!("mounting '/most-downloaded'");
    app.at("/most-downloaded")
        .get(frontend::most_downloaded::get);
    log::info!("mounting '/last-updated'");
    app.at("/last-updated").get(frontend::last_updated::get);
    log::info!("mounting '/crates/:crate'");
    app.at("/crates/:crate").get(frontend::krate::get);

    log::info!("mounting '/account/login'");
    app.at("/account/login")
        .get(frontend::account::login::get)
        .post(frontend::account::login::post);
    log::info!("mounting '/account/logout'");
    app.at("/account/logout")
        .get(frontend::account::logout::get);
    log::info!("mounting '/account/register'");
    app.at("/account/register")
        .get(frontend::account::register::get)
        .post(frontend::account::register::post);
    log::info!("mounting '/account/github'");
    app.at("/account/github")
        .get(frontend::account::github::get);
    log::info!("mounting '/account/github/attach'");
    app.at("/account/github/attach")
        .get(frontend::account::github::attach::get);
    log::info!("mounting '/account/github/detach'");
    app.at("/account/github/detach")
        .get(frontend::account::github::detach::get);
    log::info!("mounting '/account/github/callback'");
    app.at("/account/github/callback")
        .get(frontend::account::github::callback::get);
    log::info!("mounting '/account/gitlab'");
    app.at("/account/gitlab")
        .get(frontend::account::gitlab::get);
    log::info!("mounting '/account/gitlab/attach'");
    app.at("/account/gitlab/attach")
        .get(frontend::account::gitlab::attach::get);
    log::info!("mounting '/account/gitlab/detach'");
    app.at("/account/gitlab/detach")
        .get(frontend::account::gitlab::detach::get);
    log::info!("mounting '/account/gitlab/callback'");
    app.at("/account/gitlab/callback")
        .get(frontend::account::gitlab::callback::get);
    log::info!("mounting '/account/manage'");
    app.at("/account/manage")
        .get(frontend::account::manage::get);
    log::info!("mounting '/account/manage/password'");
    app.at("/account/manage/password")
        .post(frontend::account::manage::passwd::post);
    log::info!("mounting '/account/manage/tokens'");
    app.at("/account/manage/tokens")
        .post(frontend::account::manage::tokens::post);
    log::info!("mounting '/account/manage/tokens/:token-id/revoke'");
    app.at("/account/manage/tokens/:token-id/revoke")
        .get(frontend::account::manage::tokens::revoke::get);

    log::info!("mounting '/assets/*path'");
    app.at("/assets").serve_dir(frontend_config.assets.path)?;

    Ok(app)
}

fn api_routes(state: State) -> Server<State> {
    let mut app = tide::with_state(state);

    // Transform endpoint errors into the format expected by Cargo.
    app.with(After(|mut res: Response| async {
        if let Some(err) = res.error() {
            let payload = json::json!({
                "errors": [{
                    "detail": err.to_string(),
                }]
            });
            res.set_status(200);
            res.set_content_type(mime::JSON);
            res.set_body(Body::from_json(&payload)?);
        }
        Ok(res)
    }));

    log::info!("mounting '/api/v1/account/register'");
    app.at("/account/register")
        .post(api::account::register::post);
    log::info!("mounting '/api/v1/account/login'");
    app.at("/account/login").post(api::account::login::post);
    log::info!("mounting '/api/v1/account/tokens'");
    app.at("/account/tokens")
        .post(api::account::token::info::post)
        .put(api::account::token::generate::put)
        .delete(api::account::token::revoke::delete);
    log::info!("mounting '/api/v1/account/tokens/:name'");
    app.at("/account/tokens/:name")
        .get(api::account::token::info::get);
    log::info!("mounting '/api/v1/categories'");
    app.at("/categories").get(api::categories::get);
    log::info!("mounting '/api/v1/crates'");
    app.at("/crates").get(api::crates::search::get);
    log::info!("mounting '/api/v1/crates/new'");
    app.at("/crates/new").put(api::crates::publish::put);
    log::info!("mounting '/api/v1/crates/suggest'");
    app.at("/crates/suggest").get(api::crates::suggest::get);
    log::info!("mounting '/api/v1/crates/:name'");
    app.at("/crates/:name").get(api::crates::info::get);
    log::info!("mounting '/api/v1/crates/:name/owners'");
    app.at("/crates/:name/owners")
        .get(api::crates::owners::get)
        .put(api::crates::owners::put)
        .delete(api::crates::owners::delete);
    log::info!("mounting '/api/v1/crates/:name/:version/yank'");
    app.at("/crates/:name/:version/yank")
        .delete(api::crates::yank::delete);
    log::info!("mounting '/api/v1/crates/:name/:version/unyank'");
    app.at("/crates/:name/:version/unyank")
        .put(api::crates::unyank::put);
    log::info!("mounting '/api/v1/crates/:name/:version/download'");
    app.at("/crates/:name/:version/download")
        .get(api::crates::download::get);

    app
}

async fn run() -> Result<(), Error> {
    let matches = App::new("alexandrie")
        .version(build::short().as_str())
        .long_version(build::long().as_str())
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("CONFIG_FILE")
                .help("Path to the configuration file")
                .default_value("alexandrie.toml")
                .takes_value(true),
        )
        .get_matches();
    let config = matches.value_of("config").unwrap_or("alexandrie.toml");

    let contents = fs::read(config).await?;
    let config: Config = toml::from_slice(contents.as_slice())?;
    let addr = config.general.bind_address.clone();

    #[cfg(feature = "frontend")]
    let frontend_config = config.frontend.clone();

    let state: Arc<config::State> = Arc::new(config.into());

    log::info!("starting Alexandrie (version: {})", build::short());

    log::info!("running database migrations");
    #[rustfmt::skip]
    state.db.run(|conn| embedded_migrations::run(conn)).await
        .expect("migration execution error");

    let mut app = tide::with_state(Arc::clone(&state));

    log::info!("setting up request logger middleware");
    app.with(RequestLogger::new());

    #[cfg(feature = "frontend")]
    if frontend_config.enabled {
        let frontend = frontend_routes(Arc::clone(&state), frontend_config)?;
        app.at("/").nest(frontend);
    }
    app.at("/api/v1").nest(api_routes(state));

    log::info!("listening on '{0}'", addr);
    app.listen(addr).await?;

    Ok(())
}

#[async_std::main]
async fn main() {
    let _guard = logs::init();

    if let Err(err) = run().await {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}
