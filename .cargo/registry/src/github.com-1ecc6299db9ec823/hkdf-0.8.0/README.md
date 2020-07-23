# rust-hkdf [![creates.io](https://img.shields.io/crates/v/hkdf.svg)](https://crates.io/crates/hkdf) [![Documentation](https://docs.rs/hkdf/badge.svg)](https://docs.rs/hkdf)

[HMAC-based Extract-and-Expand Key Derivation Function (HKDF)](https://tools.ietf.org/html/rfc5869) for [Rust](http://www.rust-lang.org/).

Uses the Digest trait which specifies an interface common to digest functions, such as SHA-1, SHA-256, etc.

## Installation

From crates.io:

```toml
[dependencies]
hkdf = "0.7"
```

## Usage

See the example [examples/main.rs](examples/main.rs) or run it with `cargo run --example main`

## Changelog

- 0.8.0 - new API, add `Hkdf::from_prk()`, `Hkdf::extract()`
- 0.7.0 - Update digest to 0.8, refactor for API changes, remove redundant `generic-array` crate.
- 0.6.0 - remove std requirement. The `expand` signature has changed.
- 0.5.0 - removed deprecated interface, fixed omitting HKDF salt.
- 0.4.0 - RFC-inspired interface, Reduce heap allocation, remove unnecessary mut, derive Clone. deps: hex-0.3, benchmarks.
- 0.3.0 - update dependencies: digest-0.7, hmac-0.5
- 0.2.0 - support for rustc 1.20.0
- 0.1.1 - fixes to support rustc 1.5.0
- 0.1.0 - initial release

## Authors

[![Vlad Filippov](https://avatars3.githubusercontent.com/u/128755?s=70)](http://vf.io/) | [![Brian Warner](https://avatars3.githubusercontent.com/u/27146?v=4&s=70)](http://www.lothar.com/blog/) 
---|---
[Vlad Filippov](http://vf.io/) | [Brian Warner](http://www.lothar.com/blog/)
