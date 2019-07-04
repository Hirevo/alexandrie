#![feature(
    async_await,
    proc_macro_hygiene,
    decl_macro,
    custom_attribute,
    result_map_or_else,
    bind_by_move_pattern_guards
)]
#![type_length_limit = "500000000"]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;

use std::sync::{Arc, Mutex};

mod api;
pub use api::*;
mod auth;
pub use auth::*;
mod catchers;
pub use catchers::*;
mod db;
pub use db::*;
mod error;
pub use error::*;
mod index;
pub use index::*;
mod krate;
pub use krate::*;
mod state;
pub use state::*;
mod storage;
pub use storage::*;

use crate::{CLIIndex, Indexer};

fn main() -> Result<(), Error> {
    rocket::ignite()
        .mount("/api/v1", routes![api_publish, api_search, api_download])
        .register(catchers![catch_401, catch_404, catch_500])
        .attach(DbConn::fairing())
        .manage(Arc::new(Mutex::new(AppState::new(
            Index::CLIIndex(CLIIndex::new("crate-index")?),
            Storage::DiskStorage(DiskStorage::new("crate-storage")?),
        ))))
        .launch();

    Ok(())
}
