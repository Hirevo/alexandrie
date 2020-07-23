use std::cell::Cell;
use std::io::{self, IoSlice, IoSliceMut};
use std::mem;
use std::pin::Pin;
use std::slice;
use std::sync::atomic::{self, AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::task::{Context, Poll};

use futures_io::{AsyncRead, AsyncWrite};
use futures_util::task::AtomicWaker;

/// Creates a bounded single-producer single-consumer pipe.
///
/// A pipe is a ring buffer of `cap` bytes that implements traits [`AsyncRead`] and [`AsyncWrite`].
///
/// When the sender is dropped, remaining bytes in the pipe can still be read. After that, attempts
/// to read will result in `Ok(0)`, i.e. they will always 'successfully' read 0 bytes.
///
/// When the receiver is dropped, the pipe is closed and no more bytes and be written into it.
/// Further writes will result in `Ok(0)`, i.e. they will always 'successfully' write 0 bytes.
///
/// # Panics
///
/// If the capacity is 0, a panic will be thrown.
///
/// # Examples
///
/// ```
/// use futures::prelude::*;
///
/// # smol::run(async {
/// // Write a message into the pipe.
/// let (mut r, mut w) = piper::pipe(1024);
/// w.write_all(b"hello").await?;
///
/// // Close the pipe so that the read below doesn't run forever.
/// drop(w);
///
/// // Read the message.
/// let mut msg = String::new();
/// r.read_to_string(&mut msg).await?;
/// assert_eq!(msg, "hello");
/// # std::io::Result::Ok(()) });
/// ```
pub fn pipe(cap: usize) -> (Reader, Writer) {
    assert!(cap > 0, "capacity must be positive");
    assert!(cap.checked_mul(2).is_some(), "capacity is too large");

    // Allocate the ring buffer.
    let mut v = Vec::with_capacity(cap);
    let buffer = v.as_mut_ptr();
    mem::forget(v);

    let inner = Arc::new(Inner {
        head: AtomicUsize::new(0),
        tail: AtomicUsize::new(0),
        reader: AtomicWaker::new(),
        writer: AtomicWaker::new(),
        closed: AtomicBool::new(false),
        buffer,
        cap,
    });

    let r = Reader {
        inner: inner.clone(),
        head: Cell::new(0),
        tail: Cell::new(0),
    };

    let w = Writer {
        inner,
        head: Cell::new(0),
        tail: Cell::new(0),
    };

    (r, w)
}

/// The reading side of a pipe.
///
/// This struct is created by the [`pipe`] function. See its documentation for more.
///
/// # Examples
///
/// ```
/// use futures::prelude::*;
///
/// # smol::run(async {
/// let (mut r, mut w) = piper::pipe(1024);
///
/// // Write 4 bytes.
/// w.write_all(b"hello").await?;
///
/// // Read 4 bytes message.
/// let mut buf = [0u8; 4];
/// r.read_exact(&mut buf).await?;
/// # std::io::Result::Ok(()) });
/// ```
#[derive(Debug)]
pub struct Reader {
    /// The inner ring buffer.
    inner: Arc<Inner>,

    /// The head index, moved by the reader, in the range `0..2*cap`.
    ///
    /// This index always matches `inner.head`.
    head: Cell<usize>,

    /// The tail index, moved by the writer, in the range `0..2*cap`.
    ///
    /// This index is a snapshot of `index.tail` that might become stale at any point.
    tail: Cell<usize>,
}

/// The writing side of a pipe.
///
/// This struct is created by the [`pipe`] function. See its documentation for more.
///
/// # Examples
///
/// ```
/// use futures::prelude::*;
///
/// # smol::run(async {
/// let (mut r, mut w) = piper::pipe(1024);
/// w.write_all(b"hello").await?;
/// # std::io::Result::Ok(()) });
/// ```
#[derive(Debug)]
pub struct Writer {
    /// The inner ring buffer.
    inner: Arc<Inner>,

    /// The head index, moved by the reader, in the range `0..2*cap`.
    ///
    /// This index is a snapshot of `index.head` that might become stale at any point.
    head: Cell<usize>,

    /// The tail index, moved by the writer, in the range `0..2*cap`.
    ///
    /// This index always matches `inner.tail`.
    tail: Cell<usize>,
}

unsafe impl Send for Reader {}
unsafe impl Send for Writer {}

/// The inner ring buffer.
///
/// Head and tail indices are in the range `0..2*cap`, even though they really map onto the
/// `0..cap` range. The distance between head and tail indices is never more than `cap`.
///
/// The reason why indices are not in the range `0..cap` is because we need to distinguish between
/// the pipe being empty and being full. If head and tail were in `0..cap`, then `head == tail`
/// could mean the pipe is either empty or full, but we don't know which!
#[derive(Debug)]
struct Inner {
    /// The head index, moved by the reader, in the range `0..2*cap`.
    head: AtomicUsize,

    /// The tail index, moved by the writer, in the range `0..2*cap`.
    tail: AtomicUsize,

    /// A waker representing the blocked reader.
    reader: AtomicWaker,

    /// A waker representing the blocked writer.
    writer: AtomicWaker,

    /// Set to `true` if the reader or writer was dropped.
    closed: AtomicBool,

    /// The byte buffer.
    buffer: *mut u8,

    /// The buffer capacity.
    cap: usize,
}

impl Drop for Inner {
    fn drop(&mut self) {
        // Deallocate the byte buffer.
        unsafe {
            Vec::from_raw_parts(self.buffer, 0, self.cap);
        }
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        // Dropping closes the pipe and then wakes the writer.
        self.inner.closed.store(true, Ordering::SeqCst);
        self.inner.writer.wake();
    }
}

impl Drop for Writer {
    fn drop(&mut self) {
        // Dropping closes the pipe and then wakes the reader.
        self.inner.closed.store(true, Ordering::SeqCst);
        self.inner.reader.wake();
    }
}

impl AsyncRead for Reader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self).poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self).poll_read_vectored(cx, bufs)
    }
}

impl AsyncWrite for Writer {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self).poll_close(cx)
    }
}

impl AsyncRead for &Reader {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        // If the buffer is empty, we can't read any bytes.
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let mut head = self.head.get();
        let mut tail = self.tail.get();
        let cap = self.inner.cap;

        // Calculates the distance between two indices.
        let distance = |a: usize, b: usize| {
            if a <= b {
                b - a
            } else {
                2 * cap - (a - b)
            }
        };

        // If the pipe appears to be empty...
        if distance(head, tail) == 0 {
            // Reload the tail in case it's become stale.
            tail = self.inner.tail.load(Ordering::Acquire);
            self.tail.set(tail);

            // If the pipe is now really empty...
            if distance(head, tail) == 0 {
                // Register the waker.
                self.inner.reader.register(cx.waker());
                atomic::fence(Ordering::SeqCst);

                // Reload the tail after registering the waker.
                tail = self.inner.tail.load(Ordering::Acquire);
                self.tail.set(tail);

                // If the pipe is still empty...
                if distance(head, tail) == 0 {
                    // Check whether the pipe is closed or just empty.
                    if self.inner.closed.load(Ordering::Relaxed) {
                        return Poll::Ready(Ok(0));
                    } else {
                        return Poll::Pending;
                    }
                }
            }
        }

        // The pipe is not empty so remove the waker.
        self.inner.reader.take();

        // Given an index in `0..2*cap`, returns the real index in `0..cap`.
        let real_index = |i: usize| {
            if i < cap {
                i
            } else {
                i - cap
            }
        };

        // Number of bytes read so far.
        let mut count = 0;

        loop {
            // Calculate how many bytes to read in this iteration.
            let n = (16 * 1024) // Not too many bytes in one go - better to wake the writer soon!
                .min(distance(head, tail)) // No more than bytes in the pipe.
                .min(cap - real_index(head)) // Don't go past the buffer boundary.
                .min(buf.len() - count); // No more bytes than the space left in `buf`.

            // If pipe is empty or `buf` is full, return.
            if n == 0 {
                return Poll::Ready(Ok(count));
            }

            // Copy bytes from the pipe buffer into `buf`.
            let pipe_slice =
                unsafe { slice::from_raw_parts(self.inner.buffer.add(real_index(head)), n) };
            buf[count..count + n].copy_from_slice(pipe_slice);
            count += n;

            // Move the head forward.
            if head + n < 2 * cap {
                head += n;
            } else {
                head = 0;
            }

            // Store the current head index.
            self.inner.head.store(head, Ordering::Release);
            self.head.set(head);

            // Wake the writer because the pipe is not full.
            self.inner.writer.wake();
        }
    }
}

impl AsyncWrite for &Writer {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // If the pipe is empty, we can't read any bytes.
        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        // Just a quick check if the pipe is closed, which is why a relaxed load is okay.
        if self.inner.closed.load(Ordering::Relaxed) {
            return Poll::Ready(Ok(0));
        }

        // Calculates the distance between two indices.
        let cap = self.inner.cap;
        let distance = |a: usize, b: usize| {
            if a <= b {
                b - a
            } else {
                2 * cap - (a - b)
            }
        };

        let mut head = self.head.get();
        let mut tail = self.tail.get();

        // If the pipe appears to be full...
        if distance(head, tail) == cap {
            // Reload the head in case it's become stale.
            head = self.inner.head.load(Ordering::Acquire);
            self.head.set(head);

            // If the pipe is now really empty...
            if distance(head, tail) == cap {
                // Register the waker.
                self.inner.writer.register(cx.waker());
                atomic::fence(Ordering::SeqCst);

                // Reload the head after registering the waker.
                head = self.inner.head.load(Ordering::Acquire);
                self.head.set(head);

                // If the pipe is still full...
                if distance(head, tail) == cap {
                    // Check whether the pipe is closed or just full.
                    if self.inner.closed.load(Ordering::Relaxed) {
                        return Poll::Ready(Ok(0));
                    } else {
                        return Poll::Pending;
                    }
                }
            }
        }

        // The pipe is not full so remove the waker.
        self.inner.writer.take();

        // Given an index in `0..2*cap`, returns the real index in `0..cap`.
        let real_index = |i: usize| {
            if i < cap {
                i
            } else {
                i - cap
            }
        };

        // Number of bytes written so far.
        let mut count = 0;

        loop {
            // Calculate how many bytes to write in this iteration.
            let n = (16 * 1024) // Not too many bytes in one go - better to wake the reader soon!
                .min(cap - distance(head, tail)) // No more than available space in the pipe.
                .min(cap - real_index(tail)) // Don't go past the buffer boundary.
                .min(buf.len() - count); // No more bytes that is left in `buf`.

            // If the pipe is full or `buf` is empty, return.
            if n == 0 {
                return Poll::Ready(Ok(count));
            }

            // Copy bytes from `buf` into the piper buffer.
            let pipe_slice =
                unsafe { slice::from_raw_parts_mut(self.inner.buffer.add(real_index(tail)), n) };
            pipe_slice.copy_from_slice(&buf[count..count + n]);
            count += n;

            // Move the tail forward.
            if tail + n < 2 * cap {
                tail += n;
            } else {
                tail = 0;
            }

            // Store the current tail index.
            self.inner.tail.store(tail, Ordering::Release);
            self.tail.set(tail);

            // Wake the reader because the pipe is not empty.
            self.inner.reader.wake();
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }
}
