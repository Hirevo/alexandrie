# rustc-rayon

rustc-rayon is a fork of [the Rayon crate](https://github.com/rayon-rs/rayon/). It adds a few "in progress" features that rustc is using, mostly around deadlock detection. These features are not stable and should not be used by others -- though they may find their way into rayon proper at some point. In general, if you are not rustc, you should be using the real rayon crate, not rustc-rayon. =)

## License

rustc-rayon is a fork of rayon. rayon is distributed under the terms of both the MIT license and the
Apache License (Version 2.0). See [LICENSE-APACHE](LICENSE-APACHE) and
[LICENSE-MIT](LICENSE-MIT) for details. Opening a pull requests is
assumed to signal agreement with these licensing terms.
