/// Category listing endpoint (eg. "/api/v1/categories").
pub mod categories;
/// Crate downloads endpoint (eg. "/api/v1/crates/\<name\>/\<version\>/download").
pub mod download;
/// Owners management endpoint (eg. "/api/v1/crates/\<name\>/owners").
pub mod owners;
/// Publication endpoint (eg. "/api/v1/crates/new").
pub mod publish;
/// Search endpoint (eg. "/api/v1/crates?q=\<term\>").
pub mod search;
/// Crate unyanking endpoint (eg. "/api/v1/crates/\<name\>/\<version\>/unyank").
pub mod unyank;
/// Crate yanking endpoint (eg. "/api/v1/crates/\<name\>/\<version\>/yank").
pub mod yank;
