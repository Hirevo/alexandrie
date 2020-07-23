use super::{InlineAsmArch, InlineAsmType};
use rustc_macros::HashStable_Generic;
use std::fmt;

def_reg_class! {
    Arm ArmInlineAsmRegClass {
        reg,
        reg_thumb,
        sreg,
        sreg_low16,
        dreg,
        dreg_low16,
        dreg_low8,
        qreg,
        qreg_low8,
        qreg_low4,
    }
}

impl ArmInlineAsmRegClass {
    pub fn valid_modifiers(self, _arch: super::InlineAsmArch) -> &'static [char] {
        match self {
            Self::qreg | Self::qreg_low8 | Self::qreg_low4 => &['e', 'f'],
            _ => &[],
        }
    }

    pub fn suggest_class(self, _arch: InlineAsmArch, _ty: InlineAsmType) -> Option<Self> {
        None
    }

    pub fn suggest_modifier(
        self,
        _arch: InlineAsmArch,
        _ty: InlineAsmType,
    ) -> Option<(char, &'static str)> {
        None
    }

    pub fn default_modifier(self, _arch: InlineAsmArch) -> Option<(char, &'static str)> {
        None
    }

    pub fn supported_types(
        self,
        _arch: InlineAsmArch,
    ) -> &'static [(InlineAsmType, Option<&'static str>)] {
        match self {
            Self::reg | Self::reg_thumb => types! { _: I8, I16, I32, F32; },
            Self::sreg | Self::sreg_low16 => types! { "vfp2": I32, F32; },
            Self::dreg | Self::dreg_low16 | Self::dreg_low8 => types! {
                "vfp2": I64, F64, VecI8(8), VecI16(4), VecI32(2), VecI64(1), VecF32(2);
            },
            Self::qreg | Self::qreg_low8 | Self::qreg_low4 => types! {
                "neon": VecI8(16), VecI16(8), VecI32(4), VecI64(2), VecF32(4);
            },
        }
    }
}

def_regs! {
    Arm ArmInlineAsmReg ArmInlineAsmRegClass {
        r0: reg, reg_thumb = ["r0", "a1"],
        r1: reg, reg_thumb = ["r1", "a2"],
        r2: reg, reg_thumb = ["r2", "a3"],
        r3: reg, reg_thumb = ["r3", "a4"],
        r4: reg, reg_thumb = ["r4", "v1"],
        r5: reg, reg_thumb = ["r5", "v2"],
        r6: reg, reg_thumb = ["r6", "v3"],
        r7: reg, reg_thumb = ["r7", "v4"],
        r8: reg = ["r8", "v5"],
        r9: reg = ["r9", "v6", "rfp"],
        r10: reg = ["r10", "sl"],
        r12: reg = ["r12", "ip"],
        r14: reg = ["r14", "lr"],
        s0: sreg, sreg_low16 = ["s0"],
        s1: sreg, sreg_low16 = ["s1"],
        s2: sreg, sreg_low16 = ["s2"],
        s3: sreg, sreg_low16 = ["s3"],
        s4: sreg, sreg_low16 = ["s4"],
        s5: sreg, sreg_low16 = ["s5"],
        s6: sreg, sreg_low16 = ["s6"],
        s7: sreg, sreg_low16 = ["s7"],
        s8: sreg, sreg_low16 = ["s8"],
        s9: sreg, sreg_low16 = ["s9"],
        s10: sreg, sreg_low16 = ["s10"],
        s11: sreg, sreg_low16 = ["s11"],
        s12: sreg, sreg_low16 = ["s12"],
        s13: sreg, sreg_low16 = ["s13"],
        s14: sreg, sreg_low16 = ["s14"],
        s15: sreg, sreg_low16 = ["s15"],
        s16: sreg = ["s16"],
        s17: sreg = ["s17"],
        s18: sreg = ["s18"],
        s19: sreg = ["s19"],
        s20: sreg = ["s20"],
        s21: sreg = ["s21"],
        s22: sreg = ["s22"],
        s23: sreg = ["s23"],
        s24: sreg = ["s24"],
        s25: sreg = ["s25"],
        s26: sreg = ["s26"],
        s27: sreg = ["s27"],
        s28: sreg = ["s28"],
        s29: sreg = ["s29"],
        s30: sreg = ["s30"],
        s31: sreg = ["s31"],
        d0: dreg, dreg_low16, dreg_low8 = ["d0"],
        d1: dreg, dreg_low16, dreg_low8 = ["d1"],
        d2: dreg, dreg_low16, dreg_low8 = ["d2"],
        d3: dreg, dreg_low16, dreg_low8 = ["d3"],
        d4: dreg, dreg_low16, dreg_low8 = ["d4"],
        d5: dreg, dreg_low16, dreg_low8 = ["d5"],
        d6: dreg, dreg_low16, dreg_low8 = ["d6"],
        d7: dreg, dreg_low16, dreg_low8 = ["d7"],
        d8: dreg, dreg_low16 = ["d8"],
        d9: dreg, dreg_low16 = ["d9"],
        d10: dreg, dreg_low16 = ["d10"],
        d11: dreg, dreg_low16 = ["d11"],
        d12: dreg, dreg_low16 = ["d12"],
        d13: dreg, dreg_low16 = ["d13"],
        d14: dreg, dreg_low16 = ["d14"],
        d15: dreg, dreg_low16 = ["d15"],
        d16: dreg = ["d16"],
        d17: dreg = ["d17"],
        d18: dreg = ["d18"],
        d19: dreg = ["d19"],
        d20: dreg = ["d20"],
        d21: dreg = ["d21"],
        d22: dreg = ["d22"],
        d23: dreg = ["d23"],
        d24: dreg = ["d24"],
        d25: dreg = ["d25"],
        d26: dreg = ["d26"],
        d27: dreg = ["d27"],
        d28: dreg = ["d28"],
        d29: dreg = ["d29"],
        d30: dreg = ["d30"],
        d31: dreg = ["d31"],
        q0: qreg, qreg_low8, qreg_low4 = ["q0"],
        q1: qreg, qreg_low8, qreg_low4 = ["q1"],
        q2: qreg, qreg_low8, qreg_low4 = ["q2"],
        q3: qreg, qreg_low8, qreg_low4 = ["q3"],
        q4: qreg, qreg_low8 = ["q4"],
        q5: qreg, qreg_low8 = ["q5"],
        q6: qreg, qreg_low8 = ["q6"],
        q7: qreg, qreg_low8 = ["q7"],
        q8: qreg = ["q8"],
        q9: qreg = ["q9"],
        q10: qreg = ["q10"],
        q11: qreg = ["q11"],
        q12: qreg = ["q12"],
        q13: qreg = ["q13"],
        q14: qreg = ["q14"],
        q15: qreg = ["q15"],
        #error = ["r11", "fp"] =>
            "the frame pointer cannot be used as an operand for inline asm",
        #error = ["r13", "sp"] =>
            "the stack pointer cannot be used as an operand for inline asm",
        #error = ["r15", "pc"] =>
            "the program pointer cannot be used as an operand for inline asm",
    }
}

impl ArmInlineAsmReg {
    pub fn emit(
        self,
        out: &mut dyn fmt::Write,
        _arch: InlineAsmArch,
        modifier: Option<char>,
    ) -> fmt::Result {
        // Only qreg is allowed to have modifiers. This should have been
        // validated already by now.
        if let Some(modifier) = modifier {
            let index = self as u32 - Self::q0 as u32;
            assert!(index < 16);
            let index = index * 2 + (modifier == 'f') as u32;
            write!(out, "d{}", index)
        } else {
            out.write_str(self.name())
        }
    }

    pub fn overlapping_regs(self, mut cb: impl FnMut(ArmInlineAsmReg)) {
        cb(self);

        macro_rules! reg_conflicts {
            (
                $(
                    $q:ident : $d0:ident $d1:ident : $s0:ident $s1:ident $s2:ident $s3:ident
                ),*;
                $(
                    $q_high:ident : $d0_high:ident $d1_high:ident
                ),*;
            ) => {
                match self {
                    $(
                        Self::$q => {
                            cb(Self::$d0);
                            cb(Self::$d1);
                            cb(Self::$s0);
                            cb(Self::$s1);
                            cb(Self::$s2);
                            cb(Self::$s3);
                        }
                        Self::$d0 => {
                            cb(Self::$q);
                            cb(Self::$s0);
                            cb(Self::$s1);
                        }
                        Self::$d1 => {
                            cb(Self::$q);
                            cb(Self::$s2);
                            cb(Self::$s3);
                        }
                        Self::$s0 | Self::$s1 => {
                            cb(Self::$q);
                            cb(Self::$d0);
                        }
                        Self::$s2 | Self::$s3 => {
                            cb(Self::$q);
                            cb(Self::$d1);
                        }
                    )*
                    $(
                        Self::$q_high => {
                            cb(Self::$d0_high);
                            cb(Self::$d1_high);
                        }
                        Self::$d0_high | Self::$d1_high => {
                            cb(Self::$q_high);
                        }
                    )*
                    _ => {},
                }
            };
        }

        // ARM's floating-point register file is interesting in that it can be
        // viewed as 16 128-bit registers, 32 64-bit registers or 32 32-bit
        // registers. Because these views overlap, the registers of different
        // widths will conflict (e.g. d0 overlaps with s0 and s1, and q1
        // overlaps with d2 and d3).
        //
        // See section E1.3.1 of the ARM Architecture Reference Manual for
        // ARMv8-A for more details.
        reg_conflicts! {
            q0 : d0 d1 : s0 s1 s2 s3,
            q1 : d2 d3 : s4 s5 s6 s7,
            q2 : d4 d5 : s8 s9 s10 s11,
            q3 : d6 d7 : s12 s13 s14 s15,
            q4 : d8 d9 : s16 s17 s18 s19,
            q5 : d10 d11 : s20 s21 s22 s23,
            q6 : d12 d13 : s24 s25 s26 s27,
            q7 : d14 d15 : s28 s29 s30 s31;
            q8 : d16 d17,
            q9 : d18 d19,
            q10 : d20 d21,
            q11 : d22 d23,
            q12 : d24 d25,
            q13 : d26 d27,
            q14 : d28 d29,
            q15 : d30 d31;
        }
    }
}
