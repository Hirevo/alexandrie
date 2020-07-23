use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::{Request, Response, Result};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

/// An HTTP client.
#[cfg(feature = "unstable")]
#[cfg_attr(feature = "docs", doc(cfg(unstable)))]
pub trait Client: Debug + Unpin + Send + Sync + Clone + 'static {
    /// Send an HTTP request from the client.
    fn send_req(&self, req: Request) -> BoxFuture<'static, Result<Response>>;
}
