use serde::{Deserialize, Serialize};

/// The database configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
    /// The username to use when connecting to the database.
    #[cfg(any(feature = "mysql", feature = "postgres"))]
    pub user: Option<String>,
    /// The path to a file storing the password to use when connecting to the database.
    #[cfg(any(feature = "mysql", feature = "postgres"))]
    pub password_file: Option<String>,
    /// The maximum number of concurrent database connections.
    pub max_conns: Option<u32>,
}
