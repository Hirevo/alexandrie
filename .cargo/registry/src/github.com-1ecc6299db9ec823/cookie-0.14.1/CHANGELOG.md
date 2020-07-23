# Version 0.14

## Version 0.14.1 (Jun 5, 2020)

### Changes and Fixes

  * Updated `base64` dependency to 0.12.
  * Updated minimum `time` dependency to correct version: 0.2.11.
  * Added `readme` key to `Cargo.toml`, updated `license` field.

## Version 0.14.0 (May 29, 2020)

### Breaking Changes

  * The `Key::from_master()` method was deprecated in favor of the more aptly
    named `Key::derive_from()`.
  * The deprecated `CookieJar::clear()` method was removed.

### New Features

  * Added `Key::from()` to create a `Key` structure from a full-length key.
  * Signed and private cookie jars can be individually enabled via the new
    `signed` and `private` features, respectively.
  * Key derivation via key expansion can be individually enabled via the new
    `key-expansion` feature.

### General Changes and Fixes

  * `ring` is no longer a dependency: `RustCrypto`-based cryptography is used in
    lieu of `ring`. Prior to their inclusion here, the `hmac` and `hkdf` crates
    were audited.
  * Quotes, if present, are stripped from cookie values when parsing.

# Version 0.13

## Version 0.13.3 (Feb 3, 2020)

### Changes

  * The `time` dependency was unpinned from `0.2.4`, allowing any `0.2.x`
    version of `time` where `x >= 6`.

## Version 0.13.2 (Jan 28, 2020)

### Changes

  * The `time` dependency was pinned to `0.2.4` due to upstream breaking changes
    in `0.2.5`.

## Version 0.13.1 (Jan 23, 2020)

### New Features

  * Added the `CookieJar::reset_delta()` method, which reverts all _delta_
    changes to a `CookieJar`.

## Version 0.13.0 (Jan 21, 2020)

### Breaking Changes

  * `time` was updated from 0.1 to 0.2.
  * `ring` was updated from 0.14 to 0.16.
  * `SameSite::None` now writes `SameSite=None` to correspond with updated
    `SameSite` draft. `SameSite` can be unset by passing `None` to
    `Cookie::set_same_site()`.
  * `CookieBuilder` gained a lifetime: `CookieBuilder<'c>`.

### General Changes and Fixes

  * Added a CHANGELOG.
  * `expires`, `max_age`, `path`, and `domain` can be unset by passing `None` to
    the respective `Cookie::set_{field}()` method.
  * The "Expires" field is limited to a date-time of Dec 31, 9999, 23:59:59.
  * The `%` character is now properly encoded and decoded.
  * Constructor methods on `CookieBuilder` allow non-static lifetimes.
