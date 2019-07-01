#![feature(async_await, proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

use std::sync::{Arc, Mutex};

use rocket::response::Redirect;
use rocket::State;

mod krate;
pub use krate::*;
mod index;
pub use index::*;
mod error;
pub use error::*;
mod api;
pub use api::*;

pub type AppState = Arc<Mutex<Index>>;

#[get("/index")]
fn index_get(state: State<AppState>) -> Result<Redirect, Error> {
    let index = state.lock().unwrap();
    Ok(Redirect::to(index.url()?))
}

fn main() -> Result<(), Error> {
    rocket::ignite()
        .mount("/", routes![index_get])
        .mount("/api/v1", routes![
            
        ])
        .manage(Arc::new(Mutex::new(Index::new("crate-index")?)))
        .launch();

    Ok(())
}
