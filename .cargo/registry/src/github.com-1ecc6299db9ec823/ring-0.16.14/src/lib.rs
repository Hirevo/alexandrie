// Copyright 2015-2016 Brian Smith.
//
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
//
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHORS DISCLAIM ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHORS BE LIABLE FOR ANY
// SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

//! Safe, fast, small crypto using Rust with BoringSSL's cryptography
//! primitives.
//!
//! <code>git clone https://github.com/briansmith/ring</code>
//!
//! # Feature Flags
//!
//! <table>
//! <tr><th>Feature
//!     <th>Description
//! <tr><td><code>alloc (default)</code>
//!     <td>Enable features that require use of the heap, RSA in particular.
//! <tr><td><code>dev_urandom_fallback (default)</code>
//!     <td>This is only applicable to Linux. On Linux, by default,
//!         <code>ring::rand::SystemRandom</code> will fall back to reading
//!         from <code>/dev/urandom</code> if the <code>getrandom()</code>
//!         syscall isn't supported at runtime. When the
//!         <code>dev_urandom_fallback</code> feature is disabled, such
//!         fallbacks will not occur. See the documentation for
//!         <code>rand::SystemRandom</code> for more details.
//! <tr><td><code>std</code>
//!     <td>Enable features that use libstd, in particular `std::error::Error`
//!         integration.
//! <tr><td><code>wasm32_c</code>
//!     <td>Enables features that require a C compiler on wasm32 targets, such as
//!        the <code>constant_time</code> module, HMAC verification, and PBKDF2
//!        verification. Without this feature, only a subset of functionality
//!        is provided to wasm32 targets so that a C compiler isn't needed. A
//!        typical invocation would be:
//!        <code>TARGET_AR=llvm-ar cargo test --target=wasm32-unknown-unknown --features=wasm32_c</code>
//!        with <code>llvm-ar</code> and <code>clang</code> in <code>$PATH</code>.
//!        (Going forward more functionality should be enabled by default, without
//!        requiring these hacks, and without requiring a C compiler.)
//! </table>

#![doc(html_root_url = "https://briansmith.org/rustdoc/")]
#![allow(
    missing_copy_implementations,
    missing_debug_implementations,
    non_camel_case_types,
    non_snake_case,
    unsafe_code
)]
// `#[derive(...)]` uses `trivial_numeric_casts` and `unused_qualifications`
// internally.
#![deny(
    missing_docs,
    unstable_features, // Used by `internal_benches`
    unused_qualifications,
    variant_size_differences,
)]
#![forbid(
    anonymous_parameters,
    trivial_casts,
    trivial_numeric_casts,
    unused_extern_crates,
    unused_import_braces,
    unused_results,
    warnings
)]
#![no_std]
#![cfg_attr(feature = "internal_benches", allow(unstable_features), feature(test))]

#[cfg(feature = "alloc")]
extern crate alloc;

#[macro_use]
mod debug;

#[macro_use]
pub mod test;

#[macro_use]
mod arithmetic;

#[macro_use]
mod bssl;

#[macro_use]
mod polyfill;

pub mod aead;
pub mod agreement;

mod bits;

pub(crate) mod c;
pub mod constant_time;

pub mod io;

mod cpu;
pub mod digest;
mod ec;
mod endian;
pub mod error;
pub mod hkdf;
pub mod hmac;
mod limb;
pub mod pbkdf2;
pub mod pkcs8;
pub mod rand;

#[cfg(feature = "alloc")]
mod rsa;

pub mod signature;

mod sealed {
    /// Traits that are designed to only be implemented internally in *ring*.
    //
    // Usage:
    // ```
    // use crate::sealed;
    //
    // pub trait MyType: sealed::Sealed {
    //     // [...]
    // }
    //
    // impl sealed::Sealed for MyType {}
    // ```
    pub trait Sealed {}
}
