use std::env;

use log::Level;
use slog::Drain;

/// Initialises the logs mechanisms.
pub(crate) fn init() -> impl Drop {
    let decorator = slog_term::TermDecorator::new().build();
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    let drain = slog_async::Async::new(drain).build().fuse();
    let logger = slog::Logger::root(
        drain,
        slog_o!(
            "version" => env!("CARGO_PKG_VERSION"),
        ),
    );

    let guard = slog_scope::set_global_logger(logger);

    slog_stdlog::init_with_level(
        env::var("RUST_LOG")
            .map(|level| level.parse().expect("invalid log level in `${RUST_LOG}`"))
            .unwrap_or(Level::Info),
    )
    .unwrap();

    guard
}
