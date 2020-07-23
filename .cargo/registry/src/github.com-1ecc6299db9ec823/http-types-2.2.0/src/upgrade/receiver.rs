use async_std::prelude::*;
use async_std::sync;

use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::upgrade::Connection;

/// The receiving half of a channel to send an upgraded connection.
///
/// Unlike `async_std::sync::channel` the `send` method on this type can only be
/// called once, and cannot be cloned. That's because only a single instance of
/// `Connection` should be created.
#[must_use = "Futures do nothing unless polled or .awaited"]
#[derive(Debug)]
pub struct Receiver {
    receiver: sync::Receiver<Connection>,
}

impl Receiver {
    /// Create a new instance of `Receiver`.
    #[allow(unused)]
    pub(crate) fn new(receiver: sync::Receiver<Connection>) -> Self {
        Self { receiver }
    }
}

impl Future for Receiver {
    type Output = Option<Connection>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll_next(cx)
    }
}
