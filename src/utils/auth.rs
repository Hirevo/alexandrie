use diesel::prelude::*;
use futures::future::BoxFuture;
use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};
use tide::cookies::ContextExt;
use tide::middleware::{Middleware, Next};
use tide::response::IntoResponse;
use tide::{Context, Response};

use crate::db::models::Author;
use crate::db::schema::*;
use crate::error::Error;
use crate::State;

/// Session cookie's name.
pub const COOKIE_NAME: &str = "session";

/// The authentication middleware for `alexandrie`.
///
/// What it does:
///   - extracts the token from the session cookie.
///   - tries to match it with an author's session in the database.
///   - exposes an [`Author`] struct if successful.
#[derive(Clone, Default, Debug)]
pub struct AuthMiddleware {}

impl AuthMiddleware {
    /// Creates a new instance of the middleware.
    pub fn new() -> AuthMiddleware {
        AuthMiddleware {}
    }
}

impl Middleware<State> for AuthMiddleware {
    fn handle<'a>(
        &'a self,
        mut ctx: Context<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, Response> {
        Box::pin(async move {
            let now = chrono::Utc::now().naive_utc();
            let cookie = ctx.get_cookie(COOKIE_NAME).unwrap();

            if let Some(cookie) = cookie {
                let state = ctx.state();
                let repo = &state.repo;

                let author = repo.run(|conn| {
                    //? Get the non-expired session matching the user-provided token.
                    sessions::table
                        .inner_join(authors::table)
                        .select(authors::all_columns)
                        .filter(sessions::token.eq(cookie.value()))
                        .filter(sessions::expires.gt(&now))
                        .first::<Author>(conn)
                        .optional()
                });

                let author = match author.await {
                    Ok(author) => author,
                    Err(err) => return Error::from(err).into_response(),
                };

                if let Some(author) = author {
                    ctx.extensions_mut().insert(author);
                }
            }

            next.run(ctx).await
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

impl AuthExt for Context<State> {
    fn get_author(&self) -> Option<Author> {
        self.extensions().get::<Author>().cloned()
    }
    fn is_authenticated(&self) -> bool {
        self.extensions().get::<Author>().is_some()
    }
}

/// Generates a new random token (as a hex-encoded SHA-512 digest).
pub fn generate_token() -> String {
    let mut data = [0u8; 16];
    let rng = SystemRandom::new();
    rng.fill(&mut data).unwrap();
    hex::encode(hasher::digest(&hasher::SHA512, data.as_ref()))
}
