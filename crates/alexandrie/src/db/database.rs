use diesel::r2d2::{self, ConnectionManager, Pool, PooledConnection};

use crate::config::database::DatabaseConfig;

/// The database connection pool, running blocking tasks in a
/// way that does not block the async event loop.
#[derive(Debug)]
pub struct Database<T>
where
    T: diesel::Connection + 'static,
{
    connection_pool: Pool<ConnectionManager<T>>,
}

impl<T> Clone for Database<T>
where
    T: diesel::Connection + 'static,
{
    fn clone(&self) -> Self {
        Self {
            connection_pool: self.connection_pool.clone(),
        }
    }
}

impl<T> Database<T>
where
    T: diesel::Connection + 'static,
{
    /// Constructs a `Database<T>` for the given database config (creates a connection pool).
    pub fn new(database_config: &DatabaseConfig) -> Self {
        let mut builder = r2d2::Builder::default();
        if let Some(max_size) = database_config.max_conns {
            builder = builder.max_size(max_size)
        }

        #[cfg(feature = "sqlite")]
        let database_url = database_config.url.as_str();
        #[cfg(any(feature = "mysql", feature = "postgres"))]
        let database_url = {
            use url::Url;

            let mut url = Url::parse(database_config.url.as_str()).expect("invalid connection URL");
            if let Some(user) = database_config.user.as_ref() {
                if url.username().is_empty() {
                    url.set_username(user.as_str())
                        .expect("could not append username to the connection URL");
                } else {
                    panic!("conflicting usernames in database configuration");
                }
            }

            if let Some(file) = database_config.password_file.as_ref() {
                if url.password().is_none() {
                    let password = std::fs::read_to_string(file)
                        .expect("could not read from the database password file");
                    url.set_password(Some(password.as_str()))
                        .expect("could not append password to the connection URL");
                } else {
                    panic!("conflicting passwords in database configuration");
                }
            }

            url.to_string()
        };

        // this borrow is needless for sqlite, but required for mysql and postgres
        #[allow(clippy::needless_borrow)]
        Self::from_pool_builder(&database_url, builder)
    }

    /// Creates a `Database<T>` with a custom connection pool builder.
    pub fn from_pool_builder(
        database_url: &str,
        builder: diesel::r2d2::Builder<ConnectionManager<T>>,
    ) -> Self {
        let manager = ConnectionManager::new(database_url);
        let connection_pool = builder
            .build(manager)
            .expect("could not initiate test db pool");
        Database { connection_pool }
    }

    /// Runs the given closure in a way that is safe for blocking IO to the database.
    /// The closure will be passed a `Connection` from the pool to use.
    pub async fn run<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&PooledConnection<ConnectionManager<T>>) -> R + Send + 'static,
        R: Send + 'static,
        T: Send,
    {
        let pool = self.connection_pool.clone();
        let future = async_std::task::spawn_blocking(move || {
            let conn = pool.get().unwrap();
            f(&conn)
        });

        future.await
    }

    /// Runs the given closure in a way that is safe for blocking IO to the database.
    /// The closure will be passed a `Connection` from the pool to use.
    /// This closure will run in the context of a database transaction.
    /// If an error occurs, the database changes made in this closure will get rolled back to their original state.
    pub async fn transaction<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&PooledConnection<ConnectionManager<T>>) -> Result<R, E> + Send + 'static,
        T: Send,
        R: Send + 'static,
        E: From<diesel::result::Error> + Send + 'static,
    {
        let pool = self.connection_pool.clone();
        let future = async_std::task::spawn_blocking(move || {
            let conn = pool.get().unwrap();
            conn.transaction(|| f(&conn))
        });

        future.await
    }
}
