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

#[cfg(feature = "frontend")]
use rocket_contrib::serve::StaticFiles;
#[cfg(feature = "frontend")]
use rocket_contrib::templates::Template;

pub mod api;
pub mod auth;
pub mod catchers;
pub mod db;
pub mod error;
pub mod index;
pub mod krate;
pub mod state;
pub mod storage;
pub mod utils;

#[cfg(feature = "frontend")]
pub mod frontend;

use crate::db::DbConn;
use crate::error::Error;
use crate::index::cli::CLIIndex;
use crate::index::Index;
use crate::state::AppState;
use crate::storage::disk::DiskStorage;
use crate::storage::Storage;

#[cfg(feature = "frontend")]
use crate::frontend::config::Config;
#[cfg(feature = "frontend")]
use crate::utils::syntax;

fn main() -> Result<(), Error> {
    let instance = rocket::ignite();

    let instance = instance.mount(
        "/api/v1",
        routes![
            api::publish::route,
            api::search::route,
            api::download::route
        ],
    );

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

    #[cfg(feature = "frontend")]
    let instance = {
        let frontend = config
            .extras
            .get("frontend")
            .map_or_else::<Result<Config, Error>, _, _>(
                || Ok(Config::default()),
                |value| Ok(value.clone().try_into()?),
            )?;

        if frontend.enabled {
            instance
                .mount(
                    "/",
                    routes![
                        frontend::index::route,
                        frontend::search::route,
                        frontend::krate::route
                    ],
                )
                .mount("/assets", StaticFiles::from("assets"))
                .attach(Template::custom(|hbs| {
                    hbs.handlebars.register_helper(
                        "fmt_date",
                        Box::new(frontend::helpers::hbs_humanize_date),
                    );
                    hbs.handlebars.register_helper(
                        "fmt_datetime",
                        Box::new(frontend::helpers::hbs_humanize_datetime),
                    );
                    hbs.handlebars.register_helper(
                        "fmt_number",
                        Box::new(frontend::helpers::hbs_humanize_number),
                    );
                }))
                .manage(Arc::new(syntax::Config::default()))
                .manage(Arc::new(frontend))
        } else {
            instance
        }
    };

    instance
        .register(catchers![
            catchers::catch_401,
            catchers::catch_404,
            catchers::catch_500,
        ])
        .attach(DbConn::fairing())
        .manage(Arc::new(Mutex::new(AppState::new(index, storage))))
        .launch();

    Ok(())
}
