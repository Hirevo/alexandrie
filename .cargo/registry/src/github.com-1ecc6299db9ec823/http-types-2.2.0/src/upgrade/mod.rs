//! HTTP protocol upgrades.
//!
//! In HTTP it's not uncommon to convert from one protocol to another. For
//! example `HTTP/1.1` can upgrade a connection to websockets using the
//! [upgrade header](https://developer.mozilla.org/en-US/docs/Web/HTTP/Protocol_upgrade_mechanism),
//! while `HTTP/2` uses [a custom
//! handshake](https://tools.ietf.org/html/rfc8441#section-5.1). Regardless of
//! the HTTP version, changing protocols always involves some handshake,
//! after which it is turned into a stream of bytes. This module provides
//! primitives for upgrading from HTTP request-response pairs to alternate
//! protocols.

mod connection;
mod receiver;
mod sender;

pub use connection::Connection;
pub use receiver::Receiver;
pub use sender::Sender;
