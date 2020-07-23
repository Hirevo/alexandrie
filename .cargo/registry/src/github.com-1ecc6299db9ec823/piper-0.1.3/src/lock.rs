use std::cell::UnsafeCell;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::event::Event;

/// An asynchronous lock.
///
/// This type is similar to [`std::sync::Mutex`], except locking is an asynchronous operation.
///
/// Note that [`Lock`] by itself acts like an [`Arc`] in the sense that cloning it returns just
/// another reference to the same lock.
///
/// Furthermore, [`LockGuard`] is not tied to [`Lock`] by a lifetime, so you can keep guards for
/// as long as you want. This is useful when you want to spawn a task and move a guard into its
/// future.
///
/// # Examples
///
/// ```
/// # smol::run(async {
/// use piper::Lock;
/// use smol::Task;
///
/// let lock = Lock::new(0);
/// let mut tasks = vec![];
///
/// for _ in 0..10 {
///     let lock = lock.clone();
///     tasks.push(Task::spawn(async move { *lock.lock().await += 1 }));
/// }
///
/// for task in tasks {
///     task.await;
/// }
/// assert_eq!(*lock.lock().await, 10);
/// # })
/// ```
pub struct Lock<T>(Arc<Inner<T>>);

impl<T> Clone for Lock<T> {
    fn clone(&self) -> Lock<T> {
        Lock(self.0.clone())
    }
}

/// Data inside [`Lock`].
struct Inner<T> {
    /// Set to `true` when the lock is acquired by a [`LockGuard`].
    locked: AtomicBool,

    /// Lock operations waiting for the lock to be released.
    lock_ops: Event,

    /// The value inside the lock.
    data: UnsafeCell<T>,
}

unsafe impl<T: Send> Send for Lock<T> {}
unsafe impl<T: Send> Sync for Lock<T> {}

impl<T> Lock<T> {
    /// Creates a new async lock.
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Lock;
    ///
    /// let lock = Lock::new(0);
    /// ```
    pub fn new(data: T) -> Lock<T> {
        Lock(Arc::new(Inner {
            locked: AtomicBool::new(false),
            lock_ops: Event::new(),
            data: UnsafeCell::new(data),
        }))
    }

    /// Acquires the lock.
    ///
    /// Returns a guard that releases the lock when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # smol::block_on(async {
    /// use piper::Lock;
    ///
    /// let lock = Lock::new(10);
    /// let guard = lock.lock().await;
    /// assert_eq!(*guard, 10);
    /// # })
    /// ```
    pub async fn lock(&self) -> LockGuard<T> {
        loop {
            // Try acquiring the lock.
            if let Some(guard) = self.try_lock() {
                return guard;
            }

            // Start watching for notifications and try locking again.
            let listener = self.0.lock_ops.listen();
            if let Some(guard) = self.try_lock() {
                return guard;
            }
            listener.await;
        }
    }

    /// Attempts to acquire the lock.
    ///
    /// If the lock could not be acquired at this time, then [`None`] is returned. Otherwise, a
    /// guard is returned that releases the lock when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use piper::Lock;
    ///
    /// let lock = Lock::new(10);
    /// if let Some(guard) = lock.try_lock() {
    ///     assert_eq!(*guard, 10);
    /// }
    /// # ;
    /// ```
    #[inline]
    pub fn try_lock(&self) -> Option<LockGuard<T>> {
        if !self
            .0
            .locked
            .compare_and_swap(false, true, Ordering::Acquire)
        {
            Some(LockGuard(self.0.clone()))
        } else {
            None
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for Lock<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Locked;
        impl fmt::Debug for Locked {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("<locked>")
            }
        }

        match self.try_lock() {
            None => f.debug_struct("Lock").field("data", &Locked).finish(),
            Some(guard) => f.debug_struct("Lock").field("data", &&*guard).finish(),
        }
    }
}

impl<T> From<T> for Lock<T> {
    fn from(val: T) -> Lock<T> {
        Lock::new(val)
    }
}

impl<T: Default> Default for Lock<T> {
    fn default() -> Lock<T> {
        Lock::new(Default::default())
    }
}

/// A guard that releases the lock when dropped.
pub struct LockGuard<T>(Arc<Inner<T>>);

unsafe impl<T: Send> Send for LockGuard<T> {}
unsafe impl<T: Sync> Sync for LockGuard<T> {}

impl<T> Drop for LockGuard<T> {
    fn drop(&mut self) {
        self.0.locked.store(false, Ordering::Release);
        self.0.lock_ops.notify_one();
    }
}

impl<T: fmt::Debug> fmt::Debug for LockGuard<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for LockGuard<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T> Deref for LockGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T> DerefMut for LockGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}
