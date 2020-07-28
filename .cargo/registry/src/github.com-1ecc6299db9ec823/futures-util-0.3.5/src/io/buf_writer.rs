use futures_core::task::{Context, Poll};
use futures_io::{AsyncBufRead, AsyncRead, AsyncSeek, AsyncWrite, IoSlice, SeekFrom};
use pin_project::{pin_project, project};
use std::fmt;
use std::io::{self, Write};
use std::pin::Pin;
use super::DEFAULT_BUF_SIZE;

/// Wraps a writer and buffers its output.
///
/// It can be excessively inefficient to work directly with something that
/// implements [`AsyncWrite`]. A `BufWriter` keeps an in-memory buffer of data and
/// writes it to an underlying writer in large, infrequent batches.
///
/// `BufWriter` can improve the speed of programs that make *small* and
/// *repeated* write calls to the same file or network socket. It does not
/// help when writing very large amounts at once, or writing just one or a few
/// times. It also provides no advantage when writing to a destination that is
/// in memory, like a `Vec<u8>`.
///
/// When the `BufWriter` is dropped, the contents of its buffer will be
/// discarded. Creating multiple instances of a `BufWriter` on the same
/// stream can cause data loss. If you need to write out the contents of its
/// buffer, you must manually call flush before the writer is dropped.
///
/// [`AsyncWrite`]: futures_io::AsyncWrite
/// [`flush`]: super::AsyncWriteExt::flush
///
// TODO: Examples
#[pin_project]
pub struct BufWriter<W> {
    #[pin]
    inner: W,
    buf: Vec<u8>,
    written: usize,
}

impl<W: AsyncWrite> BufWriter<W> {
    /// Creates a new `BufWriter` with a default buffer capacity. The default is currently 8 KB,
    /// but may change in the future.
    pub fn new(inner: W) -> Self {
        Self::with_capacity(DEFAULT_BUF_SIZE, inner)
    }

    /// Creates a new `BufWriter` with the specified buffer capacity.
    pub fn with_capacity(cap: usize, inner: W) -> Self {
        Self {
            inner,
            buf: Vec::with_capacity(cap),
            written: 0,
        }
    }

    #[project]
    fn flush_buf(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        #[project]
        let BufWriter { mut inner, buf, written } = self.project();

        let len = buf.len();
        let mut ret = Ok(());
        while *written < len {
            match ready!(inner.as_mut().poll_write(cx, &buf[*written..])) {
                Ok(0) => {
                    ret = Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "failed to write the buffered data",
                    ));
                    break;
                }
                Ok(n) => *written += n,
                Err(e) => {
                    ret = Err(e);
                    break;
                }
            }
        }
        if *written > 0 {
            buf.drain(..*written);
        }
        *written = 0;
        Poll::Ready(ret)
    }

    delegate_access_inner!(inner, W, ());

    /// Returns a reference to the internally buffered data.
    pub fn buffer(&self) -> &[u8] {
        &self.buf
    }
}

impl<W: AsyncWrite> AsyncWrite for BufWriter<W> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        if self.buf.len() + buf.len() > self.buf.capacity() {
            ready!(self.as_mut().flush_buf(cx))?;
        }
        if buf.len() >= self.buf.capacity() {
            self.project().inner.poll_write(cx, buf)
        } else {
            Poll::Ready(self.project().buf.write(buf))
        }
    }

    fn poll_write_vectored(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        let total_len = bufs.iter().map(|b| b.len()).sum::<usize>();
        if self.buf.len() + total_len > self.buf.capacity() {
            ready!(self.as_mut().flush_buf(cx))?;
        }
        if total_len >= self.buf.capacity() {
            self.project().inner.poll_write_vectored(cx, bufs)
        } else {
            Poll::Ready(self.project().buf.write_vectored(bufs))
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        ready!(self.as_mut().flush_buf(cx))?;
        self.project().inner.poll_flush(cx)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        ready!(self.as_mut().flush_buf(cx))?;
        self.project().inner.poll_close(cx)
    }
}

impl<W: AsyncRead> AsyncRead for BufWriter<W> {
    delegate_async_read!(inner);
}

impl<W: AsyncBufRead> AsyncBufRead for BufWriter<W> {
    delegate_async_buf_read!(inner);
}

impl<W: fmt::Debug> fmt::Debug for BufWriter<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BufWriter")
            .field("writer", &self.inner)
            .field("buffer", &format_args!("{}/{}", self.buf.len(), self.buf.capacity()))
            .field("written", &self.written)
            .finish()
    }
}

impl<W: AsyncWrite + AsyncSeek> AsyncSeek for BufWriter<W> {
    /// Seek to the offset, in bytes, in the underlying writer.
    ///
    /// Seeking always writes out the internal buffer before seeking.
    fn poll_seek(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        ready!(self.as_mut().flush_buf(cx))?;
        self.project().inner.poll_seek(cx, pos)
    }
}