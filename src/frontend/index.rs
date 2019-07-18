use std::sync::Arc;

use bigdecimal::{BigDecimal, ToPrimitive};
use diesel::prelude::*;
use json::json;
use rocket::State;
use rocket_contrib::templates::Template;

use crate::db::schema::*;
use crate::db::DbConn;
use crate::error::Error;
use crate::frontend::config::Config;
use crate::frontend::helpers;

#[get("/")]
pub(crate) fn route(config: State<Arc<Config>>, conn: DbConn) -> Result<Template, Error> {
    let crate_count = crates::table
        .select(diesel::dsl::count(crates::id))
        .first::<i64>(&conn.0)?;
    let total_downloads = crates::table
        .select(diesel::dsl::sum(crates::downloads))
        .first::<Option<BigDecimal>>(&conn.0)?
        .map_or(0, |dec| {
            dec.to_u64()
                .expect("download count exceeding u64::max_value()")
        });
    let most_downloaded = crates::table
        .select((crates::name, crates::downloads))
        .order_by(crates::downloads.desc())
        .limit(10)
        .load::<(String, u64)>(&conn.0)?;
    let last_updated = crates::table
        .select((crates::name, crates::updated_at))
        .order_by(crates::updated_at.desc())
        .limit(10)
        .load::<(String, chrono::NaiveDateTime)>(&conn.0)?;
    Ok(Template::render(
        "index",
        json!({
            "instance": config.as_ref(),
            "total_downloads": helpers::humanize_number(total_downloads),
            "crate_count": helpers::humanize_number(crate_count),
            "most_downloaded": most_downloaded.into_iter().map(|(name, downloads)| json!({
                "name": name,
                "downloads": helpers::humanize_number(downloads),
            })).collect::<Vec<_>>(),
            "last_updated": last_updated.into_iter().map(|(name, date)| json!({
                "name": name,
                "updated_at": helpers::humanize_datetime(date),
            })).collect::<Vec<_>>(),
        }),
    ))
}
