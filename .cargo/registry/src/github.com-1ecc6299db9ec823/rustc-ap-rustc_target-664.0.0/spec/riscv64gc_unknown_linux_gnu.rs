use crate::spec::{CodeModel, LinkerFlavor, Target, TargetOptions, TargetResult};

pub fn target() -> TargetResult {
    Ok(Target {
        llvm_target: "riscv64-unknown-linux-gnu".to_string(),
        target_endian: "little".to_string(),
        target_pointer_width: "64".to_string(),
        target_c_int_width: "32".to_string(),
        target_env: "gnu".to_string(),
        data_layout: "e-m:e-p:64:64-i64:64-i128:128-n64-S128".to_string(),
        arch: "riscv64".to_string(),
        target_os: "linux".to_string(),
        target_vendor: "unknown".to_string(),
        linker_flavor: LinkerFlavor::Gcc,
        options: TargetOptions {
            abi_blacklist: super::riscv_base::abi_blacklist(),
            code_model: Some(CodeModel::Medium),
            cpu: "generic-rv64".to_string(),
            features: "+m,+a,+f,+d,+c".to_string(),
            llvm_abiname: "lp64d".to_string(),
            max_atomic_width: Some(64),
            ..super::linux_base::opts()
        },
    })
}
