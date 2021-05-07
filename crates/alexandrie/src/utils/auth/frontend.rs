use diesel::prelude::*;
use tide::utils::async_trait;
use tide::{Middleware, Next, Request};

use crate::db::models::Author;
use crate::db::schema::*;
use crate::State;

/// Session cookie's name.
pub const COOKIE_NAME: &str = "session";

/// The authentication middleware for `alexandrie`.
///
/// What it does:
///   - extracts the token from the session cookie.
///   - tries to match it with an author's session in the database.
///   - exposes an [`Author`] struct if successful.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct AuthMiddleware;

impl AuthMiddleware {
    /// Creates a new instance of the middleware.
    pub fn new() -> AuthMiddleware {
        AuthMiddleware {}
    }
}

#[async_trait]
impl Middleware<State> for AuthMiddleware {
    async fn handle(&self, mut req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let author_id: Option<i64> = req.session().get("author.id");

        if let Some(author_id) = author_id {
            let query = req.state().db.run(move |conn| {
                //? Get the session matching the user-provided token.
                authors::table
                    .find(author_id)
                    .first::<Author>(conn)
                    .optional()
            });

            if let Some(author) = query.await? {
                req.set_ext(author);
            }
        }

        let response = next.run(req).await;

        Ok(response)
    }
}

/// A trait to extend `tide::Request` with authentication-related helper methods.
pub trait AuthExt {
    /// Get the currently-authenticated [`Author`] (returns `None` if not authenticated).
    fn get_author(&self) -> Option<Author>;

    /// Is the user currently authenticated?
    fn is_authenticated(&self) -> bool {
        self.get_author().is_some()
    }
}

impl AuthExt for Request<State> {
    fn get_author(&self) -> Option<Author> {
        self.ext::<Author>().cloned()
    }

    fn is_authenticated(&self) -> bool {
        self.ext::<Author>().is_some()
    }
}
