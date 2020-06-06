use futures::future::BoxFuture;
use tide::{Middleware, Next, Request};

/// Middleware for logging requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RequestLogger;

impl RequestLogger {
    /// Construct a request logger.
    pub fn new() -> RequestLogger {
        RequestLogger {}
    }

    /// Log a request.
    async fn log_request<'a, State: Send + Sync + 'static>(
        self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> tide::Result {
        let path = req.uri().path().to_string();
        let method = req.method();
        info!("<-- {} {}", method, path);
        let start = std::time::Instant::now();
        let res = next.run(req).await;
        let elapsed = start.elapsed().as_millis();
        let status = match res.as_ref() {
            Ok(res) => res.status(),
            Err(err) => err.status(),
        };
        info!("--> {} {} {} {}ms", method, path, status, elapsed,);
        res
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for RequestLogger {
    fn handle<'a>(
        &'a self,
        req: Request<State>,
        next: Next<'a, State>,
    ) -> BoxFuture<'a, tide::Result> {
        Box::pin(async move { self.log_request(req, next).await })
    }
}
