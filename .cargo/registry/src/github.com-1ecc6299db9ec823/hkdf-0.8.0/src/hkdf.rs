//! An implementation of HKDF, the [HMAC-based Extract-and-Expand Key Derivation Function][1].
//!
//! # Usage
//!
//! ```rust
//! # extern crate hex;
//! # extern crate hkdf;
//! # extern crate sha2;
//!
//! # use sha2::Sha256;
//! # use hkdf::Hkdf;
//!
//! # fn main() {
//! let ikm = hex::decode("0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b").unwrap();
//! let salt = hex::decode("000102030405060708090a0b0c").unwrap();
//! let info = hex::decode("f0f1f2f3f4f5f6f7f8f9").unwrap();
//!
//! let h = Hkdf::<Sha256>::new(Some(&salt[..]), &ikm);
//! let mut okm = [0u8; 42];
//! h.expand(&info, &mut okm).unwrap();
//! println!("OKM is {}", hex::encode(&okm[..]));
//! # }
//! ```
//!
//! [1]: https://tools.ietf.org/html/rfc5869
#![no_std]

extern crate digest;
extern crate hmac;
#[cfg(feature = "std")]
extern crate std;

use core::fmt;
use digest::generic_array::{self, ArrayLength, GenericArray};
use digest::{BlockInput, FixedOutput, Input, Reset};
use hmac::{Hmac, Mac};

/// Error that is returned when supplied pseudorandom key (PRK) is not long enough.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct InvalidPrkLength;

/// Structure for InvalidLength, used for output error handling.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct InvalidLength;

/// Structure representing the HKDF, capable of HKDF-Expand and HKDF-extract operations.
#[derive(Clone)]
pub struct Hkdf<D>
where
    D: Input + BlockInput + FixedOutput + Reset + Default + Clone,
    D::BlockSize: ArrayLength<u8>,
    D::OutputSize: ArrayLength<u8>,
{
    hmac: Hmac<D>,
}

impl<D> Hkdf<D>
where
    D: Input + BlockInput + FixedOutput + Reset + Default + Clone,
    D::BlockSize: ArrayLength<u8>,
    D::OutputSize: ArrayLength<u8>,
{
    /// Convenience method for [`extract`] when the generated pseudorandom
    /// key can be ignored and only HKDF-Expand operation is needed. This is
    /// the most common constructor.
    pub fn new(salt: Option<&[u8]>, ikm: &[u8]) -> Hkdf<D> {
        let (_, hkdf) = Hkdf::extract(salt, ikm);
        hkdf
    }

    /// Create `Hkdf` from an already cryptographically strong pseudorandom key
    /// as per section 3.3 from RFC5869.
    pub fn from_prk(prk: &[u8]) -> Result<Hkdf<D>, InvalidPrkLength> {
        use generic_array::typenum::Unsigned;

        // section 2.3 specifies that prk must be "at least HashLen octets"
        if prk.len() < D::OutputSize::to_usize() {
            return Err(InvalidPrkLength);
        }

        Ok(Hkdf {
            hmac: Hmac::new_varkey(prk).expect("HMAC can take a key of any size"),
        })
    }

    /// The RFC5869 HKDF-Extract operation returning both the generated
    /// pseudorandom key and `Hkdf` struct for expanding.
    pub fn extract(salt: Option<&[u8]>, ikm: &[u8]) -> (GenericArray<u8, D::OutputSize>, Hkdf<D>) {
        let mut hmac = match salt {
            Some(s) => Hmac::<D>::new_varkey(s).expect("HMAC can take a key of any size"),
            None => Hmac::<D>::new(&Default::default()),
        };

        hmac.input(ikm);

        let prk = hmac.result().code();
        let hkdf = Hkdf::from_prk(&prk).expect("PRK size is correct");
        (prk, hkdf)
    }

    /// The RFC5869 HKDF-Expand operation
    pub fn expand(&self, info: &[u8], okm: &mut [u8]) -> Result<(), InvalidLength> {
        use generic_array::typenum::Unsigned;

        let mut prev: Option<GenericArray<u8, <D as digest::FixedOutput>::OutputSize>> = None;

        let hmac_output_bytes = D::OutputSize::to_usize();
        if okm.len() > hmac_output_bytes * 255 {
            return Err(InvalidLength);
        }

        let mut hmac = self.hmac.clone();
        for (blocknum, okm_block) in okm.chunks_mut(hmac_output_bytes).enumerate() {
            let block_len = okm_block.len();

            if let Some(ref prev) = prev {
                hmac.input(prev)
            };
            hmac.input(info);
            hmac.input(&[blocknum as u8 + 1]);

            let output = hmac.result_reset().code();
            okm_block.copy_from_slice(&output[..block_len]);

            prev = Some(output);
        }

        Ok(())
    }
}

impl fmt::Display for InvalidPrkLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str("invalid pseudorandom key length, too short")
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for InvalidPrkLength {}

impl fmt::Display for InvalidLength {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.write_str("invalid number of blocks, too large output")
    }
}

#[cfg(feature = "std")]
impl ::std::error::Error for InvalidLength {}
