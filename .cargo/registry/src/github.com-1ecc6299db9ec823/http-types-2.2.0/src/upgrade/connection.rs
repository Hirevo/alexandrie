use async_std::io::{self, prelude::*};

use std::pin::Pin;
use std::task::{Context, Poll};

/// An upgraded HTTP connection.
#[derive(Debug, Clone)]
pub struct RawConnection<Inner> {
    inner: Inner,
}

/// A boxed upgraded HTTP connection.
pub type Connection = RawConnection<Box<dyn InnerConnection + 'static>>;

/// Trait to signal the requirements for an underlying connection type.
pub trait InnerConnection: Read + Write + Send + Sync + Unpin {}
impl<T: Read + Write + Send + Sync + Unpin> InnerConnection for T {}

impl<Inner: Read + Unpin> Read for RawConnection<Inner> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_read(cx, buf)
    }
}

impl<Inner: Write + Unpin> Write for RawConnection<Inner> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut self.inner).poll_write(cx, buf)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.inner).poll_close(cx)
    }
}
