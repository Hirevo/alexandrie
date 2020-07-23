//! **GHASH**: universal hash over GF(2^128) used by AES-GCM.
//!
//! ## Implementation Notes
//!
//! The implementation of GHASH found in this crate internally uses the
//! [`polyval`] crate, which provides a similar universal hash function used by
//! AES-GCM-SIV (RFC 8452).
//!
//! By implementing GHASH in terms of POLYVAL, the two universal hash functions
//! can share a common core, meaning any optimization work (e.g. CPU-specific
//! SIMD implementations) which happens upstream in the `polyval` crate
//! benefits GHASH as well.
//!
//! From RFC 8452 Appendix A:
//! <https://tools.ietf.org/html/rfc8452#appendix-A>
//!
//! > GHASH and POLYVAL both operate in GF(2^128), although with different
//! > irreducible polynomials: POLYVAL works modulo x^128 + x^127 + x^126 +
//! > x^121 + 1 and GHASH works modulo x^128 + x^7 + x^2 + x + 1.  Note
//! > that these irreducible polynomials are the "reverse" of each other.
//!
//! [`polyval`]: https://github.com/RustCrypto/universal-hashes/tree/master/polyval

#![no_std]
#![doc(html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png")]
#![warn(missing_docs, rust_2018_idioms)]

pub use polyval::universal_hash;

use core::convert::TryInto;
use polyval::Polyval;
use universal_hash::generic_array::{typenum::U16, GenericArray};
use universal_hash::{Output, UniversalHash};
#[cfg(feature = "zeroize")]
use zeroize::Zeroize;

/// **GHASH**: universal hash over GF(2^128) used by AES-GCM.
///
/// GHASH is a universal hash function used for message authentication in
/// the AES-GCM authenticated encryption cipher.
#[derive(Clone)]
#[repr(align(16))]
pub struct GHash(Polyval);

impl UniversalHash for GHash {
    type KeySize = U16;
    type BlockSize = U16;

    /// Initialize GHASH with the given `H` field element
    fn new(h: &GenericArray<u8, U16>) -> Self {
        let mut h = *h;
        h.reverse();

        #[allow(unused_mut)]
        let mut h_polyval = mulX_POLYVAL(&h);

        #[cfg(feature = "zeroize")]
        h.zeroize();

        #[allow(clippy::let_and_return)]
        let result = GHash(Polyval::new(&h_polyval));

        #[cfg(feature = "zeroize")]
        h_polyval.zeroize();

        result
    }

    /// Input a field element `X` to be authenticated
    fn update_block(&mut self, x: &GenericArray<u8, U16>) {
        let mut x = *x;
        x.reverse();
        self.0.update_block(&x);
    }

    /// Reset internal state
    fn reset(&mut self) {
        self.0.reset();
    }

    /// Get GHASH output
    fn result(self) -> Output<U16> {
        let mut output = self.0.result().into_bytes();
        output.reverse();
        Output::new(output)
    }
}

/// The `mulX_POLYVAL()` function as defined in [RFC 8452 Appendix A][1].
/// Performs a doubling (a.k.a. "multiply by x") over GF(2^128).
/// This is useful for implementing GHASH in terms of POLYVAL.
///
/// [1]: https://tools.ietf.org/html/rfc8452#appendix-A
#[allow(non_snake_case)]
fn mulX_POLYVAL(block: &GenericArray<u8, U16>) -> GenericArray<u8, U16> {
    let mut v0 = u64::from_le_bytes(block[..8].try_into().unwrap());
    let mut v1 = u64::from_le_bytes(block[8..].try_into().unwrap());

    let v0h = v0 >> 63;
    let v1h = v1 >> 63;

    v0 <<= 1;
    v1 <<= 1;
    v0 ^= v1h;
    v1 ^= v0h ^ (v1h << 63) ^ (v1h << 62) ^ (v1h << 57);

    (u128::from(v0) | (u128::from(v1) << 64)).to_le_bytes().into()
}
