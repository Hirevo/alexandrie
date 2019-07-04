
pub mod models;
pub mod schema;

#[database("alexandrie")]
pub struct DbConn(diesel::MysqlConnection);
