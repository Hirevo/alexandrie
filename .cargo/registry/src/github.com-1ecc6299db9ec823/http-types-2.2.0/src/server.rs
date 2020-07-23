use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

use crate::{Request, Response, Result};

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a + Send>>;

/// An HTTP server.
#[cfg(feature = "unstable")]
#[cfg_attr(feature = "docs", doc(cfg(unstable)))]
pub trait Server: Debug + Unpin + Send + Sync + Clone + 'static {
    /// Receive an HTTP request on the server.
    fn recv_req(&self, req: Request) -> BoxFuture<'static, Result<Response>>;
}
