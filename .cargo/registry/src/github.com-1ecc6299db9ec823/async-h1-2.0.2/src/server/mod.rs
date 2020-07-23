//! Process HTTP connections on the server.

use std::time::Duration;

use async_std::future::{timeout, Future, TimeoutError};
use async_std::io::{self};
use async_std::io::{Read, Write};
use http_types::{Request, Response};

mod decode;
mod encode;

use decode::decode;
use encode::Encoder;

/// Configure the server.
#[derive(Debug, Clone)]
pub struct ServerOptions {
    /// Timeout to handle headers. Defaults to 60s.
    headers_timeout: Option<Duration>,
}

impl Default for ServerOptions {
    fn default() -> Self {
        Self {
            headers_timeout: Some(Duration::from_secs(60)),
        }
    }
}

/// Accept a new incoming HTTP/1.1 connection.
///
/// Supports `KeepAlive` requests by default.
pub async fn accept<RW, F, Fut>(io: RW, endpoint: F) -> http_types::Result<()>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    accept_with_opts(io, endpoint, Default::default()).await
}

/// Accept a new incoming HTTP/1.1 connection.
///
/// Supports `KeepAlive` requests by default.
pub async fn accept_with_opts<RW, F, Fut>(
    mut io: RW,
    endpoint: F,
    opts: ServerOptions,
) -> http_types::Result<()>
where
    RW: Read + Write + Clone + Send + Sync + Unpin + 'static,
    F: Fn(Request) -> Fut,
    Fut: Future<Output = http_types::Result<Response>>,
{
    loop {
        // Decode a new request, timing out if this takes longer than the timeout duration.
        let fut = decode(io.clone());

        let req = if let Some(timeout_duration) = opts.headers_timeout {
            match timeout(timeout_duration, fut).await {
                Ok(Ok(Some(r))) => r,
                Ok(Ok(None)) | Err(TimeoutError { .. }) => break, /* EOF or timeout */
                Ok(Err(e)) => return Err(e),
            }
        } else {
            match fut.await? {
                Some(r) => r,
                None => break, /* EOF */
            }
        };

        // Pass the request to the endpoint and encode the response.
        let res = endpoint(req).await?;
        let mut encoder = Encoder::new(res);

        // Stream the response to the writer.
        io::copy(&mut encoder, &mut io).await?;
    }

    Ok(())
}
