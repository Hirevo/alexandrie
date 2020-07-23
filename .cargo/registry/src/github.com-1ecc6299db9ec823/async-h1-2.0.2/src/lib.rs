//! Streaming async HTTP 1.1 parser.
//!
//! At its core HTTP is a stateful RPC protocol, where a client and server
//! communicate with one another by encoding and decoding messages between them.
//!
//! - `client` encodes HTTP requests, and decodes HTTP responses.
//! - `server` decodes HTTP requests, and encodes HTTP responses.
//!
//! A client always starts the HTTP connection. The lifetime of an HTTP
//! connection looks like this:
//!
//! ```txt
//! 1. encode            2. decode
//!         \            /
//!         -> request  ->
//!  client                server
//!         <- response <-
//!         /            \
//! 4. decode            3. encode
//! ```
//!
//! See also [`async-tls`](https://docs.rs/async-tls),
//! [`async-std`](https://docs.rs/async-std).
//!
//! # Example
//!
//! __HTTP client__
//!
//! ```no_run
//! use async_std::net::TcpStream;
//! use http_types::{Method, Request, Url};
//!
//! #[async_std::main]
//! async fn main() -> http_types::Result<()> {
//!     let stream = TcpStream::connect("127.0.0.1:8080").await?;
//!
//!     let peer_addr = stream.peer_addr()?;
//!     println!("connecting to {}", peer_addr);
//!     let url = Url::parse(&format!("http://{}/foo", peer_addr))?;
//!
//!     let req = Request::new(Method::Get, url);
//!     let res = async_h1::connect(stream.clone(), req).await?;
//!     println!("{:?}", res);
//!
//!     Ok(())
//! }
//! ```
//!
//! __HTTP Server__
//!
//! ```no_run
//! use async_std::net::{TcpStream, TcpListener};
//! use async_std::prelude::*;
//! use async_std::task;
//! use http_types::{Response, StatusCode};
//!
//! #[async_std::main]
//! async fn main() -> http_types::Result<()> {
//!     // Open up a TCP connection and create a URL.
//!     let listener = TcpListener::bind(("127.0.0.1", 8080)).await?;
//!     let addr = format!("http://{}", listener.local_addr()?);
//!     println!("listening on {}", addr);
//!
//!     // For each incoming TCP connection, spawn a task and call `accept`.
//!     let mut incoming = listener.incoming();
//!     while let Some(stream) = incoming.next().await {
//!         let stream = stream?;
//!         task::spawn(async {
//!             if let Err(err) = accept(stream).await {
//!                 eprintln!("{}", err);
//!             }
//!         });
//!     }
//!     Ok(())
//! }
//!
//! // Take a TCP stream, and convert it into sequential HTTP request / response pairs.
//! async fn accept(stream: TcpStream) -> http_types::Result<()> {
//!     println!("starting new connection from {}", stream.peer_addr()?);
//!     async_h1::accept(stream.clone(), |_req| async move {
//!         let mut res = Response::new(StatusCode::Ok);
//!         res.insert_header("Content-Type", "text/plain");
//!         res.set_body("Hello");
//!         Ok(res)
//!     })
//!     .await?;
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code, rust_2018_idioms)]
#![deny(missing_debug_implementations, nonstandard_style)]
#![warn(missing_docs, missing_doc_code_examples, unreachable_pub)]
#![cfg_attr(test, deny(warnings))]
#![allow(clippy::if_same_then_else)]
#![allow(clippy::len_zero)]
#![allow(clippy::match_bool)]
#![allow(clippy::unreadable_literal)]

/// The maximum amount of headers parsed on the server.
const MAX_HEADERS: usize = 128;

/// The maximum length of the head section we'll try to parse.
/// See: https://nodejs.org/en/blog/vulnerability/november-2018-security-releases/#denial-of-service-with-large-http-headers-cve-2018-12121
const MAX_HEAD_LENGTH: usize = 8 * 1024;

mod chunked;
mod date;
mod server;

#[doc(hidden)]
pub mod client;

pub use client::connect;
pub use server::{accept, accept_with_opts, ServerOptions};
