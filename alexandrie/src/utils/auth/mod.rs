use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(feature = "frontend")]
pub use self::frontend::*;

/// Authorization header's name.
pub const AUTHORIZATION_HEADER: &str = "authorization";

/// Generates a new random registry token (as a hex-encoded SHA-512 digest).
pub fn generate_token() -> String {
    let mut data = [0u8; 16];
    let rng = SystemRandom::new();
    rng.fill(&mut data).unwrap();
    hex::encode(hasher::digest(&hasher::SHA512, data.as_ref()))
}
