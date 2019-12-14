use serde::{Deserialize, Serialize};

/// The database configuration struct.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// The database connection URL.
    pub url: String,
    /// The maximum number of concurrent database connections.
    pub max_conns: Option<u32>,
}
