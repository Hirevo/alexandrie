dtoa
====

[![Build Status](https://api.travis-ci.org/dtolnay/dtoa.svg?branch=master)](https://travis-ci.org/dtolnay/dtoa)
[![Latest Version](https://img.shields.io/crates/v/dtoa.svg)](https://crates.io/crates/dtoa)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/dtoa)

This crate provides fast functions for printing floating-point primitives to an
[`io::Write`]. The implementation is a straightforward Rust port of [Milo Yip]'s
C++ implementation [dtoa.h]. The original C++ code of each function is included
in comments.

See also [`itoa`] for printing integer primitives.

*Version requirement: rustc 1.0+*

[`io::Write`]: https://doc.rust-lang.org/std/io/trait.Write.html
[Milo Yip]: https://github.com/miloyip
[dtoa.h]: https://github.com/miloyip/rapidjson/blob/master/include/rapidjson/internal/dtoa.h
[`itoa`]: https://github.com/dtolnay/itoa

```toml
[dependencies]
dtoa = "0.4"
```

<br>

## Performance (lower is better)

![performance](https://raw.githubusercontent.com/dtolnay/dtoa/master/performance.png)

<br>

## Examples

```rust
use std::io;

fn main() -> io::Result<()> {
    // Write to a vector or other io::Write.
    let mut buf = Vec::new();
    dtoa::write(&mut buf, 2.71828f64)?;
    println!("{:?}", buf);

    // Write to a stack buffer.
    let mut bytes = [b'\0'; 20];
    let n = dtoa::write(&mut bytes[..], 2.71828f64)?;
    println!("{:?}", &bytes[..n]);

    Ok(())
}
```

The function signature is:

```rust
fn write<W: io::Write, V: dtoa::Floating>(writer: W, value: V) -> io::Result<()>;
```

where `dtoa::Floating` is implemented for f32 and f64. The return value gives
the number of bytes written.

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
