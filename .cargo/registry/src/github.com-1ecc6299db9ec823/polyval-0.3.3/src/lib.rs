//! **POLYVAL** is a GHASH-like universal hash over GF(2^128) useful for
//! implementing [AES-GCM-SIV] or [AES-GCM/GMAC].
//!
//! From [RFC 8452 Section 3] which defines POLYVAL for use in AES-GCM_SIV:
//!
//! > "POLYVAL, like GHASH (the authenticator in AES-GCM; ...), operates in a
//! > binary field of size 2^128.  The field is defined by the irreducible
//! > polynomial x^128 + x^127 + x^126 + x^121 + 1."
//!
//! By multiplying (in the finite field sense) a sequence of 128-bit blocks of
//! input data data by a field element `H`, POLYVAL can be used to authenticate
//! the message sequence as powers (in the finite field sense) of `H`.
//!
//! ## Requirements
//!
//! - Rust 1.36.0 or newer
//! - Recommended: `RUSTFLAGS` with `-Ctarget-cpu` and `-Ctarget-feature`:
//!   - x86(-64) CPU: `target-cpu=sandybridge` or newer
//!   - SSE2 + SSE4.1: `target-feature=+sse2,+sse4.1`
//!
//! If `RUSTFLAGS` are not provided, this crate will fall back to a much slower
//! software-only implementation.
//!
//! ## Relationship to GHASH
//!
//! POLYVAL can be thought of as the little endian equivalent of GHASH, which
//! affords it a small performance advantage over GHASH when used on little
//! endian architectures.
//!
//! It has also been designed so it can also be used to compute GHASH and with
//! it GMAC, the Message Authentication Code (MAC) used by AES-GCM.
//!
//! From [RFC 8452 Appendix A]:
//!
//! > "GHASH and POLYVAL both operate in GF(2^128), although with different
//! > irreducible polynomials: POLYVAL works modulo x^128 + x^127 + x^126 +
//! > x^121 + 1 and GHASH works modulo x^128 + x^7 + x^2 + x + 1.  Note
//! > that these irreducible polynomials are the 'reverse' of each other."
//!
//! [AES-GCM-SIV]: https://en.wikipedia.org/wiki/AES-GCM-SIV
//! [AES-GCM/GMAC]: https://en.wikipedia.org/wiki/Galois/Counter_Mode
//! [RFC 8452 Section 3]: https://tools.ietf.org/html/rfc8452#section-3
//! [RFC 8452 Appendix A]: https://tools.ietf.org/html/rfc8452#appendix-A

#![no_std]
#![doc(html_logo_url = "https://raw.githubusercontent.com/RustCrypto/meta/master/logo_small.png")]
#![warn(missing_docs, rust_2018_idioms)]

mod field;

pub use universal_hash;

use universal_hash::generic_array::{typenum::U16, GenericArray};
use universal_hash::{Output, UniversalHash};

/// **POLYVAL**: GHASH-like universal hash over GF(2^128).
#[allow(non_snake_case)]
#[derive(Clone)]
#[repr(align(16))]
pub struct Polyval {
    /// GF(2^128) field element input blocks are multiplied by
    H: field::Element,

    /// Field element representing the computed universal hash
    S: field::Element,
}

impl UniversalHash for Polyval {
    type KeySize = U16;
    type BlockSize = U16;

    /// Initialize POLYVAL with the given `H` field element
    fn new(h: &GenericArray<u8, U16>) -> Self {
        Self {
            H: field::Element::from_bytes(h.clone().into()),
            S: field::Element::default(),
        }
    }

    /// Input a field element `X` to be authenticated
    fn update_block(&mut self, x: &GenericArray<u8, U16>) {
        let x = field::Element::from_bytes(x.clone().into());
        self.S = (self.S + x) * self.H;
    }

    /// Input data into the universal hash function. If the length of the
    /// data is not a multiple of the block size, the remaining data is
    /// padded with zeros up to the `BlockSize`.
    ///
    /// This approach is frequently used by AEAD modes which use
    /// Message Authentication Codes (MACs) based on universal hashing.
    fn update_padded(&mut self, data: &[u8]) {
        let mut chunks = data.chunks_exact(16);

        for chunk in &mut chunks {
            self.update_block(GenericArray::from_slice(chunk));
        }

        let rem = chunks.remainder();

        if !rem.is_empty() {
            let mut padded_block = GenericArray::default();
            padded_block[..rem.len()].copy_from_slice(rem);
            self.update_block(&padded_block);
        }
    }

    /// Reset internal state
    fn reset(&mut self) {
        self.S = field::Element::default();
    }

    /// Get POLYVAL result (i.e. computed `S` field element)
    fn result(self) -> Output<U16> {
        Output::new(GenericArray::from(self.S.to_bytes()))
    }
}
