use std::sync::{Arc, RwLock};

use cookie::{Cookie, CookieJar};
use futures::future::BoxFuture;
use tide::{Middleware, Next, Request};

/// Middleware for working with cookies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CookiesMiddleware;

impl CookiesMiddleware {
    /// Creates a new instance of the middleware.
    pub fn new() -> CookiesMiddleware {
        CookiesMiddleware {}
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for CookiesMiddleware {
    fn handle<'a>(
        &'a self,
        mut req: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, tide::Result> {
        futures::FutureExt::boxed(async move {
            let data = CookieData::from_request(&req);
            let jar = data.content.clone();
            req.set_ext(data);

            let mut res = next.run(req).await?;

            let locked = jar.read().unwrap();
            for cookie in locked.delta() {
                res.append_header("set-cookie", cookie.encoded().to_string());
            }

            Ok(res)
        })
    }
}

/// A representation of cookies which wraps a `CookieJar`.
#[derive(Debug, Clone)]
pub struct CookieData {
    /// The `CookieJar` for the current request.
    pub content: Arc<RwLock<CookieJar>>,
}

impl CookieData {
    /// Construct the cookie jar from request headers.
    pub fn from_request<State>(req: &Request<State>) -> Self {
        let mut jar = CookieJar::new();

        if let Some(headers) = req.header(tide::http::headers::COOKIE) {
            for header in headers {
                let iter = header
                    .as_str()
                    .split(';')
                    .flat_map(|value| Cookie::parse(value.trim().to_owned()));
                for cookie in iter {
                    jar.add_original(cookie);
                }
            }
        }

        CookieData {
            content: Arc::new(RwLock::new(jar)),
        }
    }
}

/// An extension to `Request` that provides cached access to cookies
pub trait CookiesExt {
    /// returns a `Cookie` by name of the cookie
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>>;

    /// Add cookie to the cookie jar
    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Option<()>;

    /// Removes the cookie. This instructs the `CookiesMiddleware` to send a cookie with empty value
    /// in the response.
    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Option<()>;
}

impl<State> CookiesExt for Request<State> {
    fn get_cookie(&self, name: &str) -> Option<Cookie<'static>> {
        let data = self.ext::<CookieData>();
        let locked = data?.content.read().unwrap();
        locked.get(name).cloned()
    }

    fn set_cookie(&mut self, cookie: Cookie<'static>) -> Option<()> {
        let data = self.ext::<CookieData>();
        let mut locked = data?.content.write().unwrap();
        locked.add(cookie);
        Some(())
    }

    fn remove_cookie(&mut self, cookie: Cookie<'static>) -> Option<()> {
        let data = self.ext::<CookieData>();
        let mut locked = data?.content.write().unwrap();
        locked.remove(cookie);
        Some(())
    }
}
