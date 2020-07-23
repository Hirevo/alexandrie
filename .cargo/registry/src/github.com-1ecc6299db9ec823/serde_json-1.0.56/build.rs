use std::env;

fn main() {
    // Decide ideal limb width for arithmetic in the float parser. Refer to
    // src/lexical/math.rs for where this has an effect.
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    match target_arch.as_str() {
        "aarch64" | "mips64" | "powerpc64" | "x86_64" => {
            println!("cargo:rustc-cfg=limb_width_64");
        }
        _ => {
            println!("cargo:rustc-cfg=limb_width_32");
        }
    }
}
