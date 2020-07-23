//! **NOTE:** This crate is DEPRECATED.
//!
//! Use the following crates instead:
//!
//! - [`async-channel`](https://docs.rs/async-channel)
//! - [`async-dup`](https://docs.rs/async-dup)
//! - [`async-lock`](https://docs.rs/async-lock)
//! - [`async-mutex`](https://docs.rs/async-mutex)
//! - [`blocking`](https://docs.rs/blocking)
//! - [`event-listener`](https://docs.rs/event-listener)

#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

mod arc;
mod chan;
mod event;
mod lock;
mod mutex;
mod pipe;

pub use arc::Arc;
pub use chan::{chan, Receiver, Sender};
pub use event::{Event, EventListener};
pub use lock::{Lock, LockGuard};
pub use mutex::{Mutex, MutexGuard};
pub use pipe::{pipe, Reader, Writer};
