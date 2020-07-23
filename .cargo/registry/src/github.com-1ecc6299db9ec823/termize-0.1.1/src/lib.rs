//! System calls for getting the terminal size.
//!
//! Getting the terminal size is performed using an ioctl command that takes
//! the file handle to the terminal -- which in this case, is stdout -- and
//! populates a structure containing the values.
//!
//! The size is needed when the user wants the output formatted into columns:
//! the default grid view, or the hybrid grid-details view.
//!
//! # Example
//!
//! To get the dimensions of your terminal window, simply use the following:
//!
//! ```no_run
//! if let Some((w, h)) = termize::dimensions() {
//!     println!("Width: {}\nHeight: {}", w, h);
//! } else {
//!     println!("Unable to get term size :(");
//! }
//! ```
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    unused_import_braces,
    unused_allocation,
    unused_qualifications,
    trivial_numeric_casts
)]

// A facade to allow exposing functions depending on the platform
mod platform;
pub use crate::platform::{dimensions, dimensions_stderr, dimensions_stdin, dimensions_stdout};
