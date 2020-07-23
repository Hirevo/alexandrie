extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap();
    if target.starts_with("wasm32") {
        // wasm32 auxilary functions are provided as a precompiled object file.
        // this is because LLVM with wasm32 support isn't widespread.
        let mut link_dir = PathBuf::new();
        link_dir.push(env!("CARGO_MANIFEST_DIR"));
        link_dir.push("src");
        link_dir.push("arch");
        link_dir.push("wasm32");

        println!("cargo:rustc-link-search={}", link_dir.display());
        println!("cargo:rustc-link-lib=stacker");
        return;
    }

    let mut cfg = cc::Build::new();
    if target.contains("windows") {
        cfg.define("WINDOWS", None);
        cfg.file("src/arch/windows.c");
        cfg.include("src/arch").compile("libstacker.a");
    } else {
        return;
    }
}
