# scoped-tls-hkt
[![CI](https://github.com/Diggsey/scoped-tls-hkt/workflows/CI/badge.svg)](https://github.com/Diggsey/scoped-tls-hkt/actions)

[Documentation](https://docs.rs/scoped-tls-hkt)

A more flexible version of `scoped-tls`, allowing the following additional
features:

- Storage of references to dynamically sized types.
- Storage of mutable references.
- Storage of types containing unbound lifetime parameters (higher-kinded types).
- Some combination of the above.

```toml
# Cargo.toml
[dependencies]
scoped-tls-hkt = "0.1"
```

# Scoped thread-local storage

A thread-local variable will appear distinct to each thread in a program. Values
stored from one thread will not be visible to other threads and vice versa.

Scoped thread-local storage builds on this concept by storing a new value into
a thread-local variable when a block of code is entered, and then restoring the
original value when execution leaves that block.

By ensuring that the original value is restored, even if the code inside the block
panics, this allows non-static data (ie. types with lifetimes, such as references)
to be temporarily (and safely) stored in a thread-local variable.

Scoped TLS is useful because it allows code deep within a program to access data
such as configuration or other contextual information set at the top level, without
having to thread additional parameters through all of the functions in-between. In
some cases, such as where a library does not provide the ability to pass through
additional data, scoped TLS can be the only viable option.

However, it should be used sparingly, because this implicitness can make it harder
to understand and debug your programs. Furthermore, access to TLS can have a
measurable performance impact, partly because some platforms do not have efficient
implementations of TLS, and partly because TLS is opaque to the compiler, so it
will often inhibit compiler optimizations that would have otherwise applied.

# License

This project is licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in scoped-tls-hkt by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
