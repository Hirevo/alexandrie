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

use std::sync::Arc;

use tokio::fs;

use axum::routing::{delete, get, post, put};
use axum::{Router, Server};
use clap::Parser;
use diesel_migrations::MigrationHarness;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

#[cfg(feature = "frontend")]
use axum::http::StatusCode;
#[cfg(feature = "frontend")]
use tower_http::services::ServeDir;
#[cfg(feature = "frontend")]
use tower_sessions::cookie::SameSite;
#[cfg(feature = "frontend")]
use tower_sessions::SessionManagerLayer;

/// API endpoints definitions.
pub mod api;
/// Configuration and internal state type definitions.
pub mod config;
/// Database abstractions module.
pub mod db;
/// Error-related type definitions.
pub mod error;
/// Various utilities and helpers.
pub mod utils;

/// Frontend endpoints definitions.
#[cfg(feature = "frontend")]
pub mod frontend;

/// Full text search
pub mod fts;

use crate::config::{AppState, Config};
use crate::utils::build;

#[cfg(feature = "frontend")]
use crate::config::FrontendConfig;
#[cfg(feature = "frontend")]
use crate::utils::sessions::SqlStore;

/// The application state type used for the web server.
pub type State = Arc<AppState>;

#[cfg(feature = "frontend")]
fn frontend_routes(state: Arc<AppState>, frontend_config: FrontendConfig) -> Router<Arc<AppState>> {
    use axum::error_handling::HandleErrorLayer;
    use axum::BoxError;
    use tower::ServiceBuilder;

    let store = SqlStore::new(state.db.clone());
    let session_layer = SessionManagerLayer::new(store)
        .with_name(frontend_config.sessions.cookie_name.as_str())
        .with_same_site(SameSite::Lax)
        .with_max_age(time::Duration::days(1));
    let session_service = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            StatusCode::BAD_REQUEST
        }))
        .layer(session_layer);

    Router::new()
        .route("/", get(frontend::index::get))
        .route("/me", get(frontend::me::get))
        .route("/search", get(frontend::search::get))
        .route("/most-downloaded", get(frontend::most_downloaded::get))
        .route("/last-updated", get(frontend::last_updated::get))
        .route("/crates/:crate", get(frontend::krate::get))
        .route(
            "/account/login",
            get(frontend::account::login::get).post(frontend::account::login::post),
        )
        .route("/account/logout", get(frontend::account::logout::get))
        .route(
            "/account/register",
            get(frontend::account::register::get).post(frontend::account::register::post),
        )
        .route("/account/github", get(frontend::account::github::get))
        .route(
            "/account/github/attach",
            get(frontend::account::github::attach::get),
        )
        .route(
            "/account/github/detach",
            get(frontend::account::github::detach::get),
        )
        .route(
            "/account/github/callback",
            get(frontend::account::github::callback::get),
        )
        .route("/account/gitlab", get(frontend::account::gitlab::get))
        .route(
            "/account/gitlab/attach",
            get(frontend::account::gitlab::attach::get),
        )
        .route(
            "/account/gitlab/detach",
            get(frontend::account::gitlab::detach::get),
        )
        .route(
            "/account/gitlab/callback",
            get(frontend::account::gitlab::callback::get),
        )
        .route("/account/manage", get(frontend::account::manage::get))
        .route(
            "/account/manage/password",
            post(frontend::account::manage::passwd::post),
        )
        .route(
            "/account/manage/tokens",
            post(frontend::account::manage::tokens::post),
        )
        .route(
            "/account/manage/tokens/:token-id/revoke",
            get(frontend::account::manage::tokens::revoke::get),
        )
        .nest_service(
            "/assets",
            ServeDir::new(frontend_config.assets.path).append_index_html_on_directories(false),
        )
        .layer(session_service)
}

fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/account/register", post(api::account::register::post))
        .route("/account/login", post(api::account::login::post))
        .route(
            "/account/tokens",
            post(api::account::token::info::post)
                .put(api::account::token::generate::put)
                .delete(api::account::token::revoke::delete),
        )
        .route("/account/tokens/:name", get(api::account::token::info::get))
        .route("/categories", get(api::categories::get))
        .route("/crates", get(api::crates::search::get))
        .route("/crates/new", put(api::crates::publish::put))
        .route("/crates/suggest", get(api::crates::suggest::get))
        .route("/crates/:name", get(api::crates::info::get))
        .route(
            "/crates/:name/owners",
            get(api::crates::owners::get)
                .put(api::crates::owners::put)
                .delete(api::crates::owners::delete),
        )
        .route(
            "/crates/:name/:version/yank",
            delete(api::crates::yank::delete),
        )
        .route(
            "/crates/:name/:version/unyank",
            put(api::crates::unyank::put),
        )
        .route(
            "/crates/:name/:version/download",
            get(api::crates::download::get),
        )
        .route("/sparse/:fst/:snd/:crate", get(api::sparse::get))
        .route("/sparse/:fst/:snd", get(api::sparse::get))
        .route("/sparse/config.json", get(api::sparse::get_config))
}

#[derive(Debug, Parser)]
#[command(about, version(build::short()), long_version(build::long()))]
struct Opts {
    /// Path to the configuration file
    #[arg(short, long, default_value = "alexandrie.toml")]
    pub config: String,
}

async fn run() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();

    tracing::info!("starting Alexandrie (version: {0})", build::short());

    let contents = fs::read_to_string(&opts.config).await?;
    let config: Config = toml::from_str(contents.as_str())?;
    let addr = config.general.bind_address.clone();

    #[cfg(feature = "frontend")]
    let frontend_config = config.frontend.clone();

    let state: AppState = config.try_into()?;

    let state = Arc::new(state);

    tracing::info!("running database migrations");
    #[rustfmt::skip]
    state.db.run(|conn| conn.run_pending_migrations(db::MIGRATIONS).map(|_| ())).await
        .expect("migration execution error");

    let database = &state.db;
    state.search.index_all(database).await?;

    let app = Router::new().nest("/api/v1", api_routes());

    #[cfg(feature = "frontend")]
    let app = if frontend_config.enabled {
        app.nest("/", frontend_routes(Arc::clone(&state), frontend_config))
    } else {
        app
    };

    let app = app
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO))
                .on_failure(trace::DefaultOnFailure::new().level(Level::ERROR)),
        )
        .with_state(Arc::clone(&state));

    tracing::info!("listening on '{addr}'");
    Server::bind(&addr.parse().unwrap())
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        // .with_target(false)
        .compact()
        .init();

    if let Err(err) = run().await {
        eprintln!("{err}");
        std::process::exit(1);
    }
}
