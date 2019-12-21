use futures::future::BoxFuture;
use tide::{Middleware, Next, Request, Response};

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
    ) -> Response {
        let path = req.uri().path().to_owned();
        let method = req.method().as_str().to_owned();
        trace!("IN => {} {}", method, path);
        let start = std::time::Instant::now();
        let res = next.run(req).await;
        let status = res.status();
        info!(
            "{} {} {} {}ms",
            method,
            path,
            status.as_str(),
            start.elapsed().as_millis()
        );
        res
    }
}

impl<State: Send + Sync + 'static> Middleware<State> for RequestLogger {
    fn handle<'a>(&'a self, req: Request<State>, next: Next<'a, State>) -> BoxFuture<'a, Response> {
        Box::pin(async move { self.log_request(req, next).await })
    }
}
