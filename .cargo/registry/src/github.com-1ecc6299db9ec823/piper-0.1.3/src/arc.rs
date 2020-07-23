use std::fmt;
use std::hash::{Hash, Hasher};
use std::io::{IoSlice, IoSliceMut};
use std::ops::Deref;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures_io::{self as io, AsyncRead, AsyncWrite};

/// A reference-counted pointer that implements async I/O traits.
///
/// This is just a wrapper around [`std::sync::Arc`] that adds the following impls:
///
/// - `impl<T> AsyncRead for Arc<T> where &T: AsyncRead {}`
/// - `impl<T> AsyncWrite for Arc<T> where &T: AsyncWrite {}`
///
/// # Examples
///
/// ```no_run
/// use futures::io;
/// use piper::Arc;
/// use smol::Async;
/// use std::net::TcpStream;
///
/// # fn main() -> std::io::Result<()> { smol::run(async {
/// // A client that echoes messages back to the server.
/// let stream = Async::<TcpStream>::connect("127.0.0.1:8000").await?;
///
/// // Create two handles to the stream.
/// let reader = Arc::new(stream);
/// let mut writer = reader.clone();
///
/// // Echo data received from the reader back into the writer.
/// io::copy(reader, &mut writer).await?;
/// # Ok(()) }) }
/// ```
pub struct Arc<T>(std::sync::Arc<T>);

impl<T> Unpin for Arc<T> {}

impl<T> Arc<T> {
    /// Constructs a new `Arc<T>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Arc;
    /// use std::sync;
    ///
    /// // These two lines are equivalent:
    /// let a = Arc::new(7);
    /// let a = Arc(sync::Arc::new(7));
    /// ```
    pub fn new(data: T) -> Arc<T> {
        Arc(std::sync::Arc::new(data))
    }
}

impl<T> Clone for Arc<T> {
    fn clone(&self) -> Arc<T> {
        Arc(self.0.clone())
    }
}

impl<T> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: fmt::Debug> fmt::Debug for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: Hash> Hash for Arc<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state)
    }
}

impl<T> fmt::Pointer for Arc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Pointer::fmt(&(&**self as *const T), f)
    }
}

impl<T: Default> Default for Arc<T> {
    fn default() -> Arc<T> {
        Arc::new(Default::default())
    }
}

impl<T> From<T> for Arc<T> {
    fn from(t: T) -> Arc<T> {
        Arc::new(t)
    }
}

// NOTE(stjepang): It would also make sense to have the following impls:
//
// - `impl<T> AsyncRead for &Arc<T> where &T: AsyncRead {}`
// - `impl<T> AsyncWrite for &Arc<T> where &T: AsyncWrite {}`
//
// However, those impls sometimes make Rust's type inference try too hard when types cannot be
// inferred. In the end, instead of complaining with a nice error message, the Rust compiler ends
// up overflowing and dumping a very long error message spanning multiple screens.
//
// Since those impls are not essential, I decided to err on the safe side and not include them.

impl<T> AsyncRead for Arc<T>
where
    for<'a> &'a T: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_read_vectored(cx, bufs)
    }
}

impl<T> AsyncWrite for Arc<T>
where
    for<'a> &'a T: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.0).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.0).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.0).poll_close(cx)
    }
}
