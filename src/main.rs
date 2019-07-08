#![feature(
    async_await,
    proc_macro_hygiene,
    decl_macro,
    custom_attribute,
    result_map_or_else,
    bind_by_move_pattern_guards
)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

use std::sync::{Arc, Mutex};

use rocket_contrib::templates::Template;

pub mod api;
pub mod auth;
pub mod catchers;
pub mod db;
pub mod error;
pub mod frontend;
pub mod index;
pub mod krate;
pub mod state;
pub mod storage;

use crate::db::DbConn;
use crate::error::Error;
use crate::frontend::config::Config;
use crate::index::cli::CLIIndex;
use crate::index::Index;
use crate::state::AppState;
use crate::storage::disk::DiskStorage;
use crate::storage::Storage;

fn main() -> Result<(), Error> {
    let instance = rocket::ignite();

    let config = instance.config();
    let index = config
        .extras
        .get("crate-index")
        .map_or_else::<Result<Index, Error>, _, _>(
            || Ok(Index::CLIIndex(CLIIndex::new("crate-index")?)),
            |value| Ok(value.clone().try_into()?),
        )?;
    let storage = config
        .extras
        .get("crate-storage")
        .map_or_else::<Result<Storage, Error>, _, _>(
            || Ok(Storage::DiskStorage(DiskStorage::new("crate-storage")?)),
            |value| Ok(value.clone().try_into()?),
        )?;
    let frontend = config
        .extras
        .get("frontend")
        .map_or_else::<Result<Config, Error>, _, _>(
            || Ok(Config::default()),
            |value| Ok(value.clone().try_into()?),
        )?;

    let instance = instance.mount(
        "/api/v1",
        routes![
            api::publish::route,
            api::search::route,
            api::download::route
        ],
    );

    let instance = if frontend.enabled {
        instance
            .mount("/", routes![frontend::index::route, frontend::search::route])
            .attach(Template::fairing())
    } else {
        instance
    };

    instance
        .register(catchers![
            catchers::catch_401,
            catchers::catch_404,
            catchers::catch_500
        ])
        .attach(DbConn::fairing())
        .manage(Arc::new(Mutex::new(AppState::new(index, storage))))
        .manage(frontend)
        .launch();

    Ok(())
}
