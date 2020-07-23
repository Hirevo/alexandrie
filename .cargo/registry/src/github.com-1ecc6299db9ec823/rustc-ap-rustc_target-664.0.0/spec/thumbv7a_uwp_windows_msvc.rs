use crate::spec::{LinkerFlavor, PanicStrategy, Target, TargetOptions, TargetResult};

pub fn target() -> TargetResult {
    let mut base = super::windows_uwp_msvc_base::opts();
    base.max_atomic_width = Some(64);
    base.has_elf_tls = true;

    // FIXME(jordanrh): use PanicStrategy::Unwind when SEH is
    // implemented for windows/arm in LLVM
    base.panic_strategy = PanicStrategy::Abort;

    Ok(Target {
        llvm_target: "thumbv7a-pc-windows-msvc".to_string(),
        target_endian: "little".to_string(),
        target_pointer_width: "32".to_string(),
        target_c_int_width: "32".to_string(),
        data_layout: "e-m:w-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64".to_string(),
        arch: "arm".to_string(),
        target_os: "windows".to_string(),
        target_env: "msvc".to_string(),
        target_vendor: "uwp".to_string(),
        linker_flavor: LinkerFlavor::Msvc,
        options: TargetOptions {
            features: "+vfp3,+neon".to_string(),
            cpu: "generic".to_string(),
            abi_blacklist: super::arm_base::abi_blacklist(),
            ..base
        },
    })
}
