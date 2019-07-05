
pub mod models;
pub mod schema;

/// Represents a database connection.  
/// Also serves as a request guard to create one.
#[database("alexandrie")]
pub struct DbConn(diesel::MysqlConnection);
