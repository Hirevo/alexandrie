use ring::digest as hasher;
use ring::rand::{SecureRandom, SystemRandom};

/// Useful authorization-related types for the programmatic API.
pub mod api;

/// Useful authorization-related types for the frontend.
#[cfg(feature = "frontend")]
pub mod frontend;

/// Generates a new random registry token (as a hex-encoded SHA-512 digest).
pub fn generate_token() -> String {
    let mut data = [0u8; 16];
    let rng = SystemRandom::new();
    rng.fill(&mut data).unwrap();
    hex::encode(hasher::digest(&hasher::SHA512, data.as_ref()))
}
