//! Async Server Sent Event parser and encoder.
//!
//! # Example
//!
//! ```
//! use async_sse::{decode, encode, Event};
//! use async_std::prelude::*;
//! use async_std::io::BufReader;
//! use async_std::task;
//!
//! #[async_std::main]
//! async fn main() -> http_types::Result<()> {
//!     // Create an encoder + sender pair and send a message.
//!     let (sender, encoder) = encode();
//!     task::spawn(async move {
//!         sender.send("cat", "chashu", None).await;
//!     });
//!
//!     // Decode messages using a decoder.
//!     let mut reader = decode(BufReader::new(encoder));
//!     let event = reader.next().await.unwrap()?;
//!     // Match and handle the event
//!
//!     # let _ = event;
//!     Ok(())
//! }
//! ```
//!
//! # References
//!
//! - [SSE Spec](https://html.spec.whatwg.org/multipage/server-sent-events.html#concept-event-stream-last-event-id)
//! - [EventSource web platform tests](https://github.com/web-platform-tests/wpt/tree/master/eventsource)

#![forbid(rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]

mod decoder;
mod encoder;
mod event;
mod handshake;
mod lines;
mod message;

pub use decoder::{decode, Decoder};
pub use encoder::{encode, Encoder, Sender};
pub use event::Event;
pub use handshake::upgrade;
pub use message::Message;

pub(crate) use lines::Lines;
