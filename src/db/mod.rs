use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};
use futures::compat::Compat01As03 as Compat;

use crate::config::database::DatabaseConfig;

/// The database models (struct representations of tables).
pub mod models;
/// The database schema definitions (in SQL types).
pub mod schema;

/// The format in which datetime records are saved in the database.
pub static DATETIME_FORMAT: &str = "%Y-%m-%d %H:%M:%S";

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

/// A database "repository", for running database workloads.
/// Manages a connection pool and running blocking tasks in a
/// way that does not block the tokio event loop.
#[derive(Debug, Clone)]
pub struct Repo<T>
where
    T: diesel::Connection + 'static,
{
    connection_pool: Pool<ConnectionManager<T>>,
}

impl<T> Repo<T>
where
    T: diesel::Connection + 'static,
{
    /// Constructs a `Repo<T>` for the given database config (creates a connection pool).
    pub fn new(database_config: &DatabaseConfig) -> Self {
        let mut builder = r2d2::Builder::default();
        if let Some(max_size) = database_config.max_conns {
            builder = builder.max_size(max_size)
        }
        Self::from_pool_builder(&database_config.url, builder)
    }

    /// Creates a `Repo<T>` with a custom connection pool builder.
    pub fn from_pool_builder(
        database_url: &str,
        builder: diesel::r2d2::Builder<ConnectionManager<T>>,
    ) -> Self {
        let manager = ConnectionManager::new(database_url);
        let connection_pool = builder
            .build(manager)
            .expect("could not initiate test db pool");
        Repo { connection_pool }
    }

    /// Runs the given closure in a way that is safe for blocking IO to the database.
    /// The closure will be passed a `Connection` from the pool to use.
    pub async fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PooledConnection<ConnectionManager<T>>) -> R + Send + std::marker::Unpin,
        T: Send,
    {
        let pool = self.connection_pool.clone();
        // `tokio_threadpool::blocking` returns a `Poll` compatible with "old style" futures.
        // `poll_fn` converts this into a future, then
        // `tokio::await` is used to convert the old style future to a `std::futures::Future`.
        // `f.take()` allows the borrow checker to be sure `f` is not moved into the inner closure
        // multiple times if `poll_fn` is called multple times.
        let mut f = Some(f);
        let future = Compat::new(futures_01::future::poll_fn(|| {
            tokio_threadpool::blocking(|| {
                let f = f.take().unwrap();
                let conn = pool.get().unwrap();
                f(&conn)
            })
            .map_err(|_| panic!("the threadpool shut down"))
        }));

        future.await.expect("Error running async database task.")
    }

    /// Runs the given closure in a way that is safe for blocking IO to the database.
    /// The closure will be passed a `Connection` from the pool to use.
    /// This closure will run in the context of a database transaction.
    /// If an error occurs, the database changes made in this closure will get rolled back to their original state.
    pub async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&PooledConnection<ConnectionManager<T>>) -> Result<R, E>
            + Send
            + std::marker::Unpin,
        T: Send,
        E: From<diesel::result::Error>,
    {
        let pool = self.connection_pool.clone();
        let mut f = Some(f);
        let future = Compat::new(futures_01::future::poll_fn(|| {
            tokio_threadpool::blocking(|| {
                let f = f.take().unwrap();
                let conn = pool.get().unwrap();
                conn.transaction(|| f(&conn))
            })
            .map_err(|_| panic!("the threadpool shut down"))
        }));

        future.await.expect("Error running async database task.")
    }
}
