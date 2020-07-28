use tide::utils::async_trait;
use tide::{Middleware, Next, Request};

/// Middleware for logging requests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct RequestLogger;

impl RequestLogger {
    /// Construct a request logger.
    pub fn new() -> RequestLogger {
        RequestLogger {}
    }
}

#[async_trait]
impl<State: Clone + Send + Sync + 'static> Middleware<State> for RequestLogger {
    async fn handle(&self, req: Request<State>, next: Next<'_, State>) -> tide::Result {
        let path = req.url().path().to_string();
        let method = req.method();
        info!("<-- {} {}", method, path);
        let start = std::time::Instant::now();
        let res = next.run(req).await;
        let elapsed = start.elapsed().as_millis();
        let status = res.status();
        let mut resp_msg = "OK".into();
        if let Some(err) = res.error() {
            resp_msg = err.to_string()
        }
        info!(
            "--> {} {} {} {}ms , {}",
            method, path, status, elapsed, resp_msg
        );
        Ok(res)
    }
}
