use std::cell::UnsafeCell;
use std::fmt;
use std::io::{IoSlice, IoSliceMut};
use std::ops::{Deref, DerefMut};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, Ordering};
use std::task::{Context, Poll};

use crate::event::Event;

use crossbeam_utils::Backoff;
use futures_io::{self as io, AsyncRead, AsyncWrite};

/// A mutex that implements async I/O traits.
///
/// This is a blocking mutex that adds the following impls:
///
/// - `impl<T> AsyncRead for Mutex<T> where &T: AsyncRead + Unpin {}`
/// - `impl<T> AsyncRead for &Mutex<T> where &T: AsyncRead + Unpin {}`
/// - `impl<T> AsyncWrite for Mutex<T> where &T: AsyncWrite + Unpin {}`
/// - `impl<T> AsyncWrite for &Mutex<T> where &T: AsyncWrite + Unpin {}`
///
/// This mutex is ensures fairness by handling lock operations in the first-in first-out order.
///
/// While primarily designed for wrapping async I/O objects, this mutex can also be used as a
/// regular blocking mutex. It's not quite as efficient as [`parking_lot::Mutex`], but it's still
/// an improvement over [`std::sync::Mutex`].
///
/// [`parking_lot::Mutex`]: https://docs.rs/parking_lot/0.10.0/parking_lot/type.Mutex.html
/// [`std::sync::Mutex`]: https://doc.rust-lang.org/std/sync/struct.Mutex.html
///
/// # Examples
///
/// ```
/// use futures::io;
/// use futures::prelude::*;
/// use piper::Mutex;
///
/// // Reads data from a stream and echoes it back.
/// async fn echo(stream: impl AsyncRead + AsyncWrite + Unpin) -> io::Result<u64> {
///     let stream = Mutex::new(stream);
///     io::copy(&stream, &mut &stream).await
/// }
/// ```
pub struct Mutex<T> {
    /// Set to `true` when the mutex is acquired by a [`MutexGuard`].
    locked: AtomicBool,

    /// Lock operations waiting for the mutex to get unlocked.
    lock_ops: Event,

    /// The value inside the mutex.
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Mutex<T> {}
unsafe impl<T: Send> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new mutex.
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// ```
    pub fn new(data: T) -> Mutex<T> {
        Mutex {
            locked: AtomicBool::new(false),
            lock_ops: Event::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Acquires the mutex, blocking the current thread until it is able to do so.
    ///
    /// Returns a guard that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// let guard = mutex.lock();
    /// assert_eq!(*guard, 10);
    /// ```
    pub fn lock(&self) -> MutexGuard<'_, T> {
        loop {
            // Try locking the mutex.
            let backoff = Backoff::new();
            loop {
                if let Some(guard) = self.try_lock() {
                    return guard;
                }
                if backoff.is_completed() {
                    break;
                }
                backoff.snooze();
            }

            // Start watching for notifications and try locking again.
            let l = self.lock_ops.listen();
            if let Some(guard) = self.try_lock() {
                return guard;
            }
            l.wait();
        }
    }

    /// Attempts to acquire the mutex.
    ///
    /// If the mutex could not be acquired at this time, then [`None`] is returned. Otherwise, a
    /// guard is returned that releases the mutex when dropped.
    ///
    /// [`None`]: https://doc.rust-lang.org/std/option/enum.Option.html#variant.None
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// if let Ok(guard) = mutex.try_lock() {
    ///     assert_eq!(*guard, 10);
    /// }
    /// # ;
    /// ```
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if !self.locked.compare_and_swap(false, true, Ordering::Acquire) {
            Some(MutexGuard(self))
        } else {
            None
        }
    }

    /// Consumes the mutex, returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// assert_eq!(mutex.into_inner(), 10);
    /// ```
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the mutex mutably, no actual locking takes place -- the mutable
    /// borrow statically guarantees the mutex is not already acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.lock(), 10);
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T: fmt::Debug> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Locked;
        impl fmt::Debug for Locked {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("<locked>")
            }
        }

        match self.try_lock() {
            None => f.debug_struct("Mutex").field("data", &Locked).finish(),
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
        }
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(val: T) -> Mutex<T> {
        Mutex::new(val)
    }
}

impl<T: Default> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for Mutex<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read_vectored(cx, bufs)
    }
}

impl<T: AsyncRead + Unpin> AsyncRead for &Mutex<T> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read(cx, buf)
    }

    fn poll_read_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &mut [IoSliceMut<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_read_vectored(cx, bufs)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for Mutex<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_close(cx)
    }
}

impl<T: AsyncWrite + Unpin> AsyncWrite for &Mutex<T> {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write(cx, buf)
    }

    fn poll_write_vectored(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        bufs: &[IoSlice<'_>],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut *self.lock()).poll_write_vectored(cx, bufs)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut *self.lock()).poll_close(cx)
    }
}

/// A guard that releases the mutex when dropped.
pub struct MutexGuard<'a, T>(&'a Mutex<T>);

unsafe impl<T: Send> Send for MutexGuard<'_, T> {}
unsafe impl<T: Sync> Sync for MutexGuard<'_, T> {}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.0.locked.store(false, Ordering::Release);
        self.0.lock_ops.notify_one();
    }
}

impl<T: fmt::Debug> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}
