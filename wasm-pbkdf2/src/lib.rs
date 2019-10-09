use std::num::NonZeroU32;

use ring::digest as hasher;
use ring::pbkdf2;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn pbkdf2_encode(password: &str, salt: &str, iteration_count: u32) -> String {
    let mut out = [0u8; hasher::SHA512_OUTPUT_LEN];
    let iteration_count = NonZeroU32::new(iteration_count).expect("invalid iteration count");
    pbkdf2::derive(
        pbkdf2::PBKDF2_HMAC_SHA512,
        iteration_count,
        salt.as_bytes(),
        password.as_bytes(),
        &mut out,
    );
    hex::encode(out.as_ref())
}
