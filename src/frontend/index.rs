use json::json;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::frontend::config::Config;

#[get("/")]
pub(crate) fn route(config: State<Config>) -> Template {
    Template::render(
        "index",
        json!({
            "instance": config.inner(),
        }),
    )
}
