/// Token information endpoint (eg. "/api/v1/account/token/info").
pub mod info;
/// Token revocation endpoint (eg. "/api/v1/account/token/revoke").
pub mod revoke;

pub use self::info::post;
pub use self::revoke::delete;
