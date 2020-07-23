extern crate cc;

fn find_assembly(arch: &str, endian: &str, os: &str, env: &str) -> Option<(&'static str, bool)> {
    match (arch, endian, os, env) {
        // The implementations for stack switching exist, but, officially, doing so without Fibers
        // is not supported in Windows. For x86_64 the implementation actually works locally,
        // but failed tests in CI (???). Might want to have a feature for experimental support
        // here.
        ("x86", _, "windows", "msvc") => Some(("src/arch/x86_msvc.asm", false)),
        ("x86_64", _, "windows", "msvc") => Some(("src/arch/x86_64_msvc.asm", false)),
        ("arm", _, "windows", "msvc") => Some(("src/arch/arm_armasm.asm", false)),
        ("aarch64", _, "windows", "msvc") => Some(("src/arch/aarch64_armasm.asm", false)),
        ("x86", _, "windows", _) => Some(("src/arch/x86_windows_gnu.s", false)),
        ("x86_64", _, "windows", _) => Some(("src/arch/x86_64_windows_gnu.s", false)),

        ("x86", _, _, _) => Some(("src/arch/x86.s", true)),
        ("x86_64", _, _, _) => Some(("src/arch/x86_64.s", true)),
        ("arm", _, _, _) => Some(("src/arch/arm_aapcs.s", true)),
        ("aarch64", _, _, _) => Some(("src/arch/aarch_aapcs64.s", true)),
        ("powerpc", _, _, _) => Some(("src/arch/powerpc32.s", true)),
        ("powerpc64", "little", _, _) => Some(("src/arch/powerpc64_openpower.s", true)),
        ("powerpc64", _, _, _) => Some(("src/arch/powerpc64.s", true)),
        ("s390x", _, _, _) => Some(("src/arch/zseries_linux.s", true)),
        ("mips", _, _, _) => Some(("src/arch/mips_eabi.s", true)),
        ("mips64", _, _, _) => Some(("src/arch/mips64_eabi.s", true)),
        ("sparc64", _, _, _) => Some(("src/arch/sparc64.s", true)),
        ("sparc", _, _, _) => Some(("src/arch/sparc_sysv.s", true)),
        ("riscv32", _, _, _) => Some(("src/arch/riscv.s", true)),
        ("riscv64", _, _, _) => Some(("src/arch/riscv64.s", true)),

        // The implementations exist, but the compilers are unlikely be able to work with this.
        // ("wasm32",     _,        "unknown", _) => Some(("src/arch/wasm32.s", false)),
        // ("wasm32",     _,        "wasi",    _) => Some(("src/arch/wasm32.s", false)),
        _ => None,
    }
}

fn main() {
    let arch = ::std::env::var("CARGO_CFG_TARGET_ARCH").unwrap();
    let env = ::std::env::var("CARGO_CFG_TARGET_ENV").unwrap();
    let os = ::std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let endian = ::std::env::var("CARGO_CFG_TARGET_ENDIAN").unwrap();
    let asm = if let Some((asm, canswitch)) = find_assembly(&arch, &endian, &os, &env) {
        println!("cargo:rustc-cfg=asm");
        if canswitch {
            println!("cargo:rustc-cfg=switchable_stack")
        }
        asm
    } else {
        println!(
            "cargo:warning=Target {}-{}-{} has no assembly files!",
            arch, os, env
        );
        return;
    };

    let mut cfg = cc::Build::new();
    let msvc = cfg.get_compiler().is_like_msvc();

    if !msvc {
        cfg.flag("-xassembler-with-cpp");
        cfg.define(&*format!("CFG_TARGET_OS_{}", os), None);
        cfg.define(&*format!("CFG_TARGET_ARCH_{}", arch), None);
        cfg.define(&*format!("CFG_TARGET_ENV_{}", env), None);
    }
    cfg.file(asm);

    cfg.compile("libpsm_s.a");
}
