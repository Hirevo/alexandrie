# POLYVAL: fast universal hash function and MAC

[![crate][crate-image]][crate-link]
[![Docs][docs-image]][docs-link]
![Apache2/MIT licensed][license-image]
![Rust Version][rustc-image]
[![Build Status][build-image]][build-link]

[POLYVAL][1] ([RFC 8452][2]) is a [universal hash function][3] which operates
over GF(2^128) and can be used for constructing a
[Message Authentication Code (MAC)][4].

Its primary intended use is for implementing [AES-GCM-SIV][5], however it is
closely related to [GHASH][6] and therefore can also be used to implement
[AES-GCM][7] at no cost to performance on little endian architectures.

[Documentation][docs-link]

## Security Warning

No security audits of this crate have ever been performed, and it has not been
thoroughly assessed to ensure its operation is constant-time on common CPU
architectures.

USE AT YOUR OWN RISK!

## License

Licensed under either of:

 * [Apache License, Version 2.0](http://www.apache.org/licenses/LICENSE-2.0)
 * [MIT license](http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

[//]: # (badges)

[crate-image]: https://img.shields.io/crates/v/polyval.svg
[crate-link]: https://crates.io/crates/polyval
[docs-image]: https://docs.rs/polyval/badge.svg
[docs-link]: https://docs.rs/polyval/
[license-image]: https://img.shields.io/badge/license-Apache2.0/MIT-blue.svg
[rustc-image]: https://img.shields.io/badge/rustc-1.36+-blue.svg
[build-image]: https://travis-ci.com/RustCrypto/universal-hashes.svg?branch=master
[build-link]: https://travis-ci.com/RustCrypto/universal-hashes

[//]: # (general links)

[1]: https://en.wikipedia.org/wiki/AES-GCM-SIV#Operation
[2]: https://tools.ietf.org/html/rfc8452#section-3
[3]: https://en.wikipedia.org/wiki/Universal_hashing
[4]: https://en.wikipedia.org/wiki/Message_authentication_code
[5]: https://en.wikipedia.org/wiki/AES-GCM-SIV
[6]: https://en.wikipedia.org/wiki/Galois/Counter_Mode#Mathematical_basis
[7]: https://en.wikipedia.org/wiki/Galois/Counter_Mode
