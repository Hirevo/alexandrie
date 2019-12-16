use diesel::prelude::*;
use futures::future::BoxFuture;
use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};
use tide::{IntoResponse, Middleware, Next, Request, Response};

use crate::db::models::Author;
use crate::db::schema::*;
use crate::db::DATETIME_FORMAT;
use crate::error::Error;
use crate::utils::cookies::CookiesExt;
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

impl Middleware<State> for AuthMiddleware {
    fn handle<'a>(
        &'a self,
        mut req: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Response> {
        futures::FutureExt::boxed(async move {
            let now = chrono::Utc::now().naive_utc();
            let cookie = req.get_cookie(COOKIE_NAME);

            if let Some(cookie) = cookie {
                let state = req.state().clone();
                let repo = &state.repo;
                let query = repo.run(move |conn| {
                    //? Get the session matching the user-provided token.
                    sessions::table
                        .inner_join(authors::table)
                        .select((sessions::expires, authors::all_columns))
                        .filter(sessions::token.eq(cookie.value()))
                        .first::<(String, Author)>(conn)
                        .optional()
                });

                let results = match query.await {
                    Ok(results) => results,
                    Err(err) => return Error::from(err).into_response(),
                };

                if let Some((expires, author)) = results {
                    let expires =
                        chrono::NaiveDateTime::parse_from_str(expires.as_str(), DATETIME_FORMAT)
                            .unwrap();

                    if expires > now {
                        req = req.set_local(author);
                    }
                }
            }

            next.run(req).await
        })
    }
}

/// A trait to extend `Context` with authentication-related helper methods.
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
        self.local::<Author>().cloned()
    }

    fn is_authenticated(&self) -> bool {
        self.local::<Author>().is_some()
    }
}

/// Generates a new random token (as a hex-encoded SHA-512 digest).
pub fn generate_token() -> String {
    let mut data = [0u8; 16];
    let rng = SystemRandom::new();
    rng.fill(&mut data).unwrap();
    hex::encode(hasher::digest(&hasher::SHA512, data.as_ref()))
}
