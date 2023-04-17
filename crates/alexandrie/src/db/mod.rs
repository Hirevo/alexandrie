use diesel_migrations::EmbeddedMigrations;

/// The database connection pool implementation.
pub mod database;
/// The database models (struct representations of tables).
pub mod models;
/// The database schema definitions (in SQL types).
pub mod schema;

/// The SQL migrations (for MySQL/MariaDB).
#[cfg(feature = "mysql")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations/mysql");
/// The SQL migrations (for SQLite).
#[cfg(feature = "sqlite")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations/sqlite");
/// The SQL migrations (for PostgreSQL).
#[cfg(feature = "postgres")]
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations/postgres");

/// The format in which datetime records are saved in the database.
pub static DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

/// The main database type (parameterized with the selected connection type).
pub type Database = self::database::Database<Connection>;

/// The connection type (MySQL database).
#[cfg(feature = "mysql")]
pub type Connection = diesel::mysql::MysqlConnection;

/// The connection type (SQLite database).
#[cfg(feature = "sqlite")]
pub type Connection = diesel::sqlite::SqliteConnection;

/// The connection type (PostgreSQL database).
#[cfg(feature = "postgres")]
pub type Connection = diesel::pg::PgConnection;

#[cfg(not(any(feature = "mysql", feature = "sqlite", feature = "postgres")))]
compile_error!("At least one database backend must be enabled to build this crate (eg. by passing argument `--features [mysql|sqlite|postgres]`).");
#[cfg(not(any(feature = "mysql", feature = "sqlite", feature = "postgres")))]
pub type Connection = unimplemented!();

#[cfg(any(
    all(feature = "mysql", feature = "sqlite"),
    all(feature = "mysql", feature = "postgres"),
    all(feature = "sqlite", feature = "postgres")
))]
compile_error!("Only one database backend can be enabled at a time.");
