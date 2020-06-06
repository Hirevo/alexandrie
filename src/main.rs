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
#[macro_use]
extern crate log;
#[macro_use(slog_o)]
extern crate slog;

use std::sync::Arc;

use async_std::fs;

/// API endpoints definitions.
pub mod api;
/// Configuration and internal state type definitions.
pub mod config;
/// Database abstractions module.
pub mod db;
/// Error-related type definitions.
pub mod error;
/// Crate index management strategies.
pub mod index;
/// Logs initialisation.
pub mod logs;
/// Crate storage strategies.
pub mod storage;
/// Various utilities and helpers.
pub mod utils;

/// Frontend endpoints definitions.
#[cfg(feature = "frontend")]
pub mod frontend;

use crate::config::Config;
use crate::error::Error;
use crate::utils::request_log::RequestLogger;

#[cfg(feature = "frontend")]
use crate::utils::auth::AuthMiddleware;
#[cfg(feature = "frontend")]
use crate::utils::cookies::CookiesMiddleware;

/// The instantiated [`crate::db::Repo`] type alias.
pub type Repo = db::Repo<db::Connection>;

/// The application state type used for the web server.
pub type State = Arc<config::State>;

#[cfg(feature = "mysql")]
embed_migrations!("migrations/mysql");
#[cfg(feature = "sqlite")]
embed_migrations!("migrations/sqlite");
#[cfg(feature = "postgres")]
embed_migrations!("migrations/postgres");

#[allow(clippy::cognitive_complexity)]
async fn run() -> Result<(), Error> {
    let _guard = logs::init();

    let contents = fs::read("alexandrie.toml").await?;
    let config: Config = toml::from_slice(contents.as_slice())?;
    let addr = format!("{0}:{1}", config.general.addr, config.general.port);

    #[cfg(feature = "frontend")]
    let frontend_enabled = config.frontend.enabled;

    let state: config::State = config.into();

    info!("running database migrations");
    #[rustfmt::skip]
    state.repo.run(|conn| embedded_migrations::run(conn)).await
        .expect("migration execution error");

    let mut app = tide::with_state(Arc::new(state));

    info!("setting up request logger middleware");
    app.middleware(RequestLogger::new());

    #[cfg(feature = "frontend")]
    {
        if frontend_enabled {
            info!("setting up cookie middleware");
            app.middleware(CookiesMiddleware::new());
            info!("setting up authentication middleware");
            app.middleware(AuthMiddleware::new());

            info!("mounting '/'");
            app.at("/").get(frontend::index::get);
            info!("mounting '/me'");
            app.at("/me").get(frontend::me::get);
            info!("mounting '/search'");
            app.at("/search").get(frontend::search::get);
            info!("mounting '/most-downloaded'");
            app.at("/most-downloaded")
                .get(frontend::most_downloaded::get);
            info!("mounting '/last-updated'");
            app.at("/last-updated").get(frontend::last_updated::get);
            info!("mounting '/crates/:crate'");
            app.at("/crates/:crate").get(frontend::krate::get);

            info!("mounting '/account/login'");
            app.at("/account/login")
                .get(frontend::account::login::get)
                .post(frontend::account::login::post);
            info!("mounting '/account/logout'");
            app.at("/account/logout")
                .get(frontend::account::logout::get);
            info!("mounting '/account/register'");
            app.at("/account/register")
                .get(frontend::account::register::get)
                .post(frontend::account::register::post);
            info!("mounting '/account/manage'");
            app.at("/account/manage")
                .get(frontend::account::manage::get);
            info!("mounting '/account/manage/password'");
            app.at("/account/manage/password")
                .post(frontend::account::manage::passwd::post);
            info!("mounting '/account/manage/tokens'");
            app.at("/account/manage/tokens")
                .post(frontend::account::manage::tokens::post);
            info!("mounting '/account/manage/tokens/:token-id/revoke'");
            app.at("/account/manage/tokens/:token-id/revoke")
                .get(frontend::account::manage::tokens::revoke::get);

            info!("mounting '/assets/*path'");
            app.at("/assets").serve_dir("assets")?;
        }
    }

    info!("mounting '/api/v1/account/register'");
    app.at("/api/v1/account/register")
        .post(api::account::register::post);
    info!("mounting '/api/v1/account/login'");
    app.at("/api/v1/account/login")
        .post(api::account::login::post);
    info!("mounting '/api/v1/account/tokens'");
    app.at("/api/v1/account/tokens")
        .post(api::account::token::info::post)
        .put(api::account::token::generate::put)
        .delete(api::account::token::revoke::delete);
    info!("mounting '/api/v1/account/tokens/:name'");
    app.at("/api/v1/account/tokens/:name")
        .get(api::account::token::info::get);
    info!("mounting '/api/v1/categories'");
    app.at("/api/v1/categories").get(api::categories::get);
    info!("mounting '/api/v1/crates'");
    app.at("/api/v1/crates").get(api::crates::search::get);
    info!("mounting '/api/v1/crates/new'");
    app.at("/api/v1/crates/new").put(api::crates::publish::put);
    info!("mounting '/api/v1/crates/suggest'");
    app.at("/api/v1/crates/suggest")
        .get(api::crates::suggest::get);
    info!("mounting '/api/v1/crates/:name'");
    app.at("/api/v1/crates/:name").get(api::crates::info::get);
    info!("mounting '/api/v1/crates/:name/owners'");
    app.at("/api/v1/crates/:name/owners")
        .get(api::crates::owners::get)
        .put(api::crates::owners::put)
        .delete(api::crates::owners::delete);
    info!("mounting '/api/v1/crates/:name/:version/yank'");
    app.at("/api/v1/crates/:name/:version/yank")
        .delete(api::crates::yank::delete);
    info!("mounting '/api/v1/crates/:name/:version/unyank'");
    app.at("/api/v1/crates/:name/:version/unyank")
        .put(api::crates::unyank::put);
    info!("mounting '/api/v1/crates/:name/:version/download'");
    app.at("/api/v1/crates/:name/:version/download")
        .get(api::crates::download::get);

    info!("listening on {0}", addr);
    app.listen(addr).await?;

    Ok(())
}

#[async_std::main]
async fn main() {
    if let Err(err) = run().await {
        eprintln!("{}", err);
    }
}
