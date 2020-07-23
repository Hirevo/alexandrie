// Targets the Cortex-M23 processor (Baseline ARMv8-M)

use crate::spec::{LinkerFlavor, LldFlavor, Target, TargetOptions, TargetResult};

pub fn target() -> TargetResult {
    Ok(Target {
        llvm_target: "thumbv8m.base-none-eabi".to_string(),
        target_endian: "little".to_string(),
        target_pointer_width: "32".to_string(),
        target_c_int_width: "32".to_string(),
        data_layout: "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64".to_string(),
        arch: "arm".to_string(),
        target_os: "none".to_string(),
        target_env: String::new(),
        target_vendor: String::new(),
        linker_flavor: LinkerFlavor::Lld(LldFlavor::Ld),

        options: TargetOptions {
            // ARMv8-M baseline doesn't support unaligned loads/stores so we disable them
            // with +strict-align.
            features: "+strict-align".to_string(),
            max_atomic_width: Some(32),
            ..super::thumb_base::opts()
        },
    })
}
