use crate::abi::{self, Abi, Align, FieldsShape, Size};
use crate::abi::{HasDataLayout, LayoutOf, TyAndLayout, TyAndLayoutMethods};
use crate::spec::{self, HasTargetSpec};

mod aarch64;
mod amdgpu;
mod arm;
mod hexagon;
mod mips;
mod mips64;
mod msp430;
mod nvptx;
mod nvptx64;
mod powerpc;
mod powerpc64;
mod riscv;
mod s390x;
mod sparc;
mod sparc64;
mod wasm32;
mod wasm32_bindgen_compat;
mod x86;
mod x86_64;
mod x86_win64;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PassMode {
    /// Ignore the argument.
    Ignore,
    /// Pass the argument directly.
    Direct(ArgAttributes),
    /// Pass a pair's elements directly in two arguments.
    Pair(ArgAttributes, ArgAttributes),
    /// Pass the argument after casting it, to either
    /// a single uniform or a pair of registers.
    Cast(CastTarget),
    /// Pass the argument indirectly via a hidden pointer.
    /// The second value, if any, is for the extra data (vtable or length)
    /// which indicates that it refers to an unsized rvalue.
    Indirect(ArgAttributes, Option<ArgAttributes>),
}

// Hack to disable non_upper_case_globals only for the bitflags! and not for the rest
// of this module
pub use attr_impl::ArgAttribute;

#[allow(non_upper_case_globals)]
#[allow(unused)]
mod attr_impl {
    // The subset of llvm::Attribute needed for arguments, packed into a bitfield.
    bitflags::bitflags! {
        #[derive(Default)]
        pub struct ArgAttribute: u16 {
            const ByVal     = 1 << 0;
            const NoAlias   = 1 << 1;
            const NoCapture = 1 << 2;
            const NonNull   = 1 << 3;
            const ReadOnly  = 1 << 4;
            const SExt      = 1 << 5;
            const StructRet = 1 << 6;
            const ZExt      = 1 << 7;
            const InReg     = 1 << 8;
        }
    }
}

/// A compact representation of LLVM attributes (at least those relevant for this module)
/// that can be manipulated without interacting with LLVM's Attribute machinery.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct ArgAttributes {
    pub regular: ArgAttribute,
    /// The minimum size of the pointee, guaranteed to be valid for the duration of the whole call
    /// (corresponding to LLVM's dereferenceable and dereferenceable_or_null attributes).
    pub pointee_size: Size,
    pub pointee_align: Option<Align>,
}

impl ArgAttributes {
    pub fn new() -> Self {
        ArgAttributes {
            regular: ArgAttribute::default(),
            pointee_size: Size::ZERO,
            pointee_align: None,
        }
    }

    pub fn set(&mut self, attr: ArgAttribute) -> &mut Self {
        self.regular |= attr;
        self
    }

    pub fn contains(&self, attr: ArgAttribute) -> bool {
        self.regular.contains(attr)
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum RegKind {
    Integer,
    Float,
    Vector,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct Reg {
    pub kind: RegKind,
    pub size: Size,
}

macro_rules! reg_ctor {
    ($name:ident, $kind:ident, $bits:expr) => {
        pub fn $name() -> Reg {
            Reg { kind: RegKind::$kind, size: Size::from_bits($bits) }
        }
    };
}

impl Reg {
    reg_ctor!(i8, Integer, 8);
    reg_ctor!(i16, Integer, 16);
    reg_ctor!(i32, Integer, 32);
    reg_ctor!(i64, Integer, 64);
    reg_ctor!(i128, Integer, 128);

    reg_ctor!(f32, Float, 32);
    reg_ctor!(f64, Float, 64);
}

impl Reg {
    pub fn align<C: HasDataLayout>(&self, cx: &C) -> Align {
        let dl = cx.data_layout();
        match self.kind {
            RegKind::Integer => match self.size.bits() {
                1 => dl.i1_align.abi,
                2..=8 => dl.i8_align.abi,
                9..=16 => dl.i16_align.abi,
                17..=32 => dl.i32_align.abi,
                33..=64 => dl.i64_align.abi,
                65..=128 => dl.i128_align.abi,
                _ => panic!("unsupported integer: {:?}", self),
            },
            RegKind::Float => match self.size.bits() {
                32 => dl.f32_align.abi,
                64 => dl.f64_align.abi,
                _ => panic!("unsupported float: {:?}", self),
            },
            RegKind::Vector => dl.vector_align(self.size).abi,
        }
    }
}

/// An argument passed entirely registers with the
/// same kind (e.g., HFA / HVA on PPC64 and AArch64).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Uniform {
    pub unit: Reg,

    /// The total size of the argument, which can be:
    /// * equal to `unit.size` (one scalar/vector),
    /// * a multiple of `unit.size` (an array of scalar/vectors),
    /// * if `unit.kind` is `Integer`, the last element
    ///   can be shorter, i.e., `{ i64, i64, i32 }` for
    ///   64-bit integers with a total size of 20 bytes.
    pub total: Size,
}

impl From<Reg> for Uniform {
    fn from(unit: Reg) -> Uniform {
        Uniform { unit, total: unit.size }
    }
}

impl Uniform {
    pub fn align<C: HasDataLayout>(&self, cx: &C) -> Align {
        self.unit.align(cx)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct CastTarget {
    pub prefix: [Option<RegKind>; 8],
    pub prefix_chunk: Size,
    pub rest: Uniform,
}

impl From<Reg> for CastTarget {
    fn from(unit: Reg) -> CastTarget {
        CastTarget::from(Uniform::from(unit))
    }
}

impl From<Uniform> for CastTarget {
    fn from(uniform: Uniform) -> CastTarget {
        CastTarget { prefix: [None; 8], prefix_chunk: Size::ZERO, rest: uniform }
    }
}

impl CastTarget {
    pub fn pair(a: Reg, b: Reg) -> CastTarget {
        CastTarget {
            prefix: [Some(a.kind), None, None, None, None, None, None, None],
            prefix_chunk: a.size,
            rest: Uniform::from(b),
        }
    }

    pub fn size<C: HasDataLayout>(&self, cx: &C) -> Size {
        (self.prefix_chunk * self.prefix.iter().filter(|x| x.is_some()).count() as u64)
            .align_to(self.rest.align(cx))
            + self.rest.total
    }

    pub fn align<C: HasDataLayout>(&self, cx: &C) -> Align {
        self.prefix
            .iter()
            .filter_map(|x| x.map(|kind| Reg { kind, size: self.prefix_chunk }.align(cx)))
            .fold(cx.data_layout().aggregate_align.abi.max(self.rest.align(cx)), |acc, align| {
                acc.max(align)
            })
    }
}

/// Return value from the `homogeneous_aggregate` test function.
#[derive(Copy, Clone, Debug)]
pub enum HomogeneousAggregate {
    /// Yes, all the "leaf fields" of this struct are passed in the
    /// same way (specified in the `Reg` value).
    Homogeneous(Reg),

    /// There are no leaf fields at all.
    NoData,
}

/// Error from the `homogeneous_aggregate` test function, indicating
/// there are distinct leaf fields passed in different ways,
/// or this is uninhabited.
#[derive(Copy, Clone, Debug)]
pub struct Heterogeneous;

impl HomogeneousAggregate {
    /// If this is a homogeneous aggregate, returns the homogeneous
    /// unit, else `None`.
    pub fn unit(self) -> Option<Reg> {
        match self {
            HomogeneousAggregate::Homogeneous(reg) => Some(reg),
            HomogeneousAggregate::NoData => None,
        }
    }

    /// Try to combine two `HomogeneousAggregate`s, e.g. from two fields in
    /// the same `struct`. Only succeeds if only one of them has any data,
    /// or both units are identical.
    fn merge(self, other: HomogeneousAggregate) -> Result<HomogeneousAggregate, Heterogeneous> {
        match (self, other) {
            (x, HomogeneousAggregate::NoData) | (HomogeneousAggregate::NoData, x) => Ok(x),

            (HomogeneousAggregate::Homogeneous(a), HomogeneousAggregate::Homogeneous(b)) => {
                if a != b {
                    return Err(Heterogeneous);
                }
                Ok(self)
            }
        }
    }
}

impl<'a, Ty> TyAndLayout<'a, Ty> {
    fn is_aggregate(&self) -> bool {
        match self.abi {
            Abi::Uninhabited | Abi::Scalar(_) | Abi::Vector { .. } => false,
            Abi::ScalarPair(..) | Abi::Aggregate { .. } => true,
        }
    }

    /// Returns `Homogeneous` if this layout is an aggregate containing fields of
    /// only a single type (e.g., `(u32, u32)`). Such aggregates are often
    /// special-cased in ABIs.
    ///
    /// Note: We generally ignore fields of zero-sized type when computing
    /// this value (see #56877).
    ///
    /// This is public so that it can be used in unit tests, but
    /// should generally only be relevant to the ABI details of
    /// specific targets.
    pub fn homogeneous_aggregate<C>(&self, cx: &C) -> Result<HomogeneousAggregate, Heterogeneous>
    where
        Ty: TyAndLayoutMethods<'a, C> + Copy,
        C: LayoutOf<Ty = Ty, TyAndLayout = Self>,
    {
        match self.abi {
            Abi::Uninhabited => Err(Heterogeneous),

            // The primitive for this algorithm.
            Abi::Scalar(ref scalar) => {
                let kind = match scalar.value {
                    abi::Int(..) | abi::Pointer => RegKind::Integer,
                    abi::F32 | abi::F64 => RegKind::Float,
                };
                Ok(HomogeneousAggregate::Homogeneous(Reg { kind, size: self.size }))
            }

            Abi::Vector { .. } => {
                assert!(!self.is_zst());
                Ok(HomogeneousAggregate::Homogeneous(Reg {
                    kind: RegKind::Vector,
                    size: self.size,
                }))
            }

            Abi::ScalarPair(..) | Abi::Aggregate { .. } => {
                // Helper for computing `homogeneous_aggregate`, allowing a custom
                // starting offset (used below for handling variants).
                let from_fields_at =
                    |layout: Self,
                     start: Size|
                     -> Result<(HomogeneousAggregate, Size), Heterogeneous> {
                        let is_union = match layout.fields {
                            FieldsShape::Primitive => {
                                unreachable!("aggregates can't have `FieldsShape::Primitive`")
                            }
                            FieldsShape::Array { count, .. } => {
                                assert_eq!(start, Size::ZERO);

                                let result = if count > 0 {
                                    layout.field(cx, 0).homogeneous_aggregate(cx)?
                                } else {
                                    HomogeneousAggregate::NoData
                                };
                                return Ok((result, layout.size));
                            }
                            FieldsShape::Union(_) => true,
                            FieldsShape::Arbitrary { .. } => false,
                        };

                        let mut result = HomogeneousAggregate::NoData;
                        let mut total = start;

                        for i in 0..layout.fields.count() {
                            if !is_union && total != layout.fields.offset(i) {
                                return Err(Heterogeneous);
                            }

                            let field = layout.field(cx, i);

                            result = result.merge(field.homogeneous_aggregate(cx)?)?;

                            // Keep track of the offset (without padding).
                            let size = field.size;
                            if is_union {
                                total = total.max(size);
                            } else {
                                total += size;
                            }
                        }

                        Ok((result, total))
                    };

                let (mut result, mut total) = from_fields_at(*self, Size::ZERO)?;

                match &self.variants {
                    abi::Variants::Single { .. } => {}
                    abi::Variants::Multiple { variants, .. } => {
                        // Treat enum variants like union members.
                        // HACK(eddyb) pretend the `enum` field (discriminant)
                        // is at the start of every variant (otherwise the gap
                        // at the start of all variants would disqualify them).
                        //
                        // NB: for all tagged `enum`s (which include all non-C-like
                        // `enum`s with defined FFI representation), this will
                        // match the homogeneous computation on the equivalent
                        // `struct { tag; union { variant1; ... } }` and/or
                        // `union { struct { tag; variant1; } ... }`
                        // (the offsets of variant fields should be identical
                        // between the two for either to be a homogeneous aggregate).
                        let variant_start = total;
                        for variant_idx in variants.indices() {
                            let (variant_result, variant_total) =
                                from_fields_at(self.for_variant(cx, variant_idx), variant_start)?;

                            result = result.merge(variant_result)?;
                            total = total.max(variant_total);
                        }
                    }
                }

                // There needs to be no padding.
                if total != self.size {
                    Err(Heterogeneous)
                } else {
                    match result {
                        HomogeneousAggregate::Homogeneous(_) => {
                            assert_ne!(total, Size::ZERO);
                        }
                        HomogeneousAggregate::NoData => {
                            assert_eq!(total, Size::ZERO);
                        }
                    }
                    Ok(result)
                }
            }
        }
    }
}

/// Information about how to pass an argument to,
/// or return a value from, a function, under some ABI.
#[derive(Debug)]
pub struct ArgAbi<'a, Ty> {
    pub layout: TyAndLayout<'a, Ty>,

    /// Dummy argument, which is emitted before the real argument.
    pub pad: Option<Reg>,

    pub mode: PassMode,
}

impl<'a, Ty> ArgAbi<'a, Ty> {
    pub fn new(layout: TyAndLayout<'a, Ty>) -> Self {
        ArgAbi { layout, pad: None, mode: PassMode::Direct(ArgAttributes::new()) }
    }

    pub fn make_indirect(&mut self) {
        assert_eq!(self.mode, PassMode::Direct(ArgAttributes::new()));

        // Start with fresh attributes for the pointer.
        let mut attrs = ArgAttributes::new();

        // For non-immediate arguments the callee gets its own copy of
        // the value on the stack, so there are no aliases. It's also
        // program-invisible so can't possibly capture
        attrs.set(ArgAttribute::NoAlias).set(ArgAttribute::NoCapture).set(ArgAttribute::NonNull);
        attrs.pointee_size = self.layout.size;
        // FIXME(eddyb) We should be doing this, but at least on
        // i686-pc-windows-msvc, it results in wrong stack offsets.
        // attrs.pointee_align = Some(self.layout.align.abi);

        let extra_attrs = self.layout.is_unsized().then_some(ArgAttributes::new());

        self.mode = PassMode::Indirect(attrs, extra_attrs);
    }

    pub fn make_indirect_byval(&mut self) {
        self.make_indirect();
        match self.mode {
            PassMode::Indirect(ref mut attrs, _) => {
                attrs.set(ArgAttribute::ByVal);
            }
            _ => unreachable!(),
        }
    }

    pub fn extend_integer_width_to(&mut self, bits: u64) {
        // Only integers have signedness
        if let Abi::Scalar(ref scalar) = self.layout.abi {
            if let abi::Int(i, signed) = scalar.value {
                if i.size().bits() < bits {
                    if let PassMode::Direct(ref mut attrs) = self.mode {
                        attrs.set(if signed { ArgAttribute::SExt } else { ArgAttribute::ZExt });
                    }
                }
            }
        }
    }

    pub fn cast_to<T: Into<CastTarget>>(&mut self, target: T) {
        assert_eq!(self.mode, PassMode::Direct(ArgAttributes::new()));
        self.mode = PassMode::Cast(target.into());
    }

    pub fn pad_with(&mut self, reg: Reg) {
        self.pad = Some(reg);
    }

    pub fn is_indirect(&self) -> bool {
        match self.mode {
            PassMode::Indirect(..) => true,
            _ => false,
        }
    }

    pub fn is_sized_indirect(&self) -> bool {
        match self.mode {
            PassMode::Indirect(_, None) => true,
            _ => false,
        }
    }

    pub fn is_unsized_indirect(&self) -> bool {
        match self.mode {
            PassMode::Indirect(_, Some(_)) => true,
            _ => false,
        }
    }

    pub fn is_ignore(&self) -> bool {
        match self.mode {
            PassMode::Ignore => true,
            _ => false,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Conv {
    // General language calling conventions, for which every target
    // should have its own backend (e.g. LLVM) support.
    C,
    Rust,

    // Target-specific calling conventions.
    ArmAapcs,

    Msp430Intr,

    PtxKernel,

    X86Fastcall,
    X86Intr,
    X86Stdcall,
    X86ThisCall,
    X86VectorCall,

    X86_64SysV,
    X86_64Win64,

    AmdGpuKernel,
}

/// Metadata describing how the arguments to a native function
/// should be passed in order to respect the native ABI.
///
/// I will do my best to describe this structure, but these
/// comments are reverse-engineered and may be inaccurate. -NDM
#[derive(Debug)]
pub struct FnAbi<'a, Ty> {
    /// The LLVM types of each argument.
    pub args: Vec<ArgAbi<'a, Ty>>,

    /// LLVM return type.
    pub ret: ArgAbi<'a, Ty>,

    pub c_variadic: bool,

    /// The count of non-variadic arguments.
    ///
    /// Should only be different from args.len() when c_variadic is true.
    /// This can be used to know whether an argument is variadic or not.
    pub fixed_count: usize,

    pub conv: Conv,

    pub can_unwind: bool,
}

impl<'a, Ty> FnAbi<'a, Ty> {
    pub fn adjust_for_cabi<C>(&mut self, cx: &C, abi: spec::abi::Abi) -> Result<(), String>
    where
        Ty: TyAndLayoutMethods<'a, C> + Copy,
        C: LayoutOf<Ty = Ty, TyAndLayout = TyAndLayout<'a, Ty>> + HasDataLayout + HasTargetSpec,
    {
        match &cx.target_spec().arch[..] {
            "x86" => {
                let flavor = if abi == spec::abi::Abi::Fastcall {
                    x86::Flavor::Fastcall
                } else {
                    x86::Flavor::General
                };
                x86::compute_abi_info(cx, self, flavor);
            }
            "x86_64" => {
                if abi == spec::abi::Abi::SysV64 {
                    x86_64::compute_abi_info(cx, self);
                } else if abi == spec::abi::Abi::Win64 || cx.target_spec().options.is_like_windows {
                    x86_win64::compute_abi_info(self);
                } else {
                    x86_64::compute_abi_info(cx, self);
                }
            }
            "aarch64" => aarch64::compute_abi_info(cx, self),
            "amdgpu" => amdgpu::compute_abi_info(cx, self),
            "arm" => arm::compute_abi_info(cx, self),
            "mips" => mips::compute_abi_info(cx, self),
            "mips64" => mips64::compute_abi_info(cx, self),
            "powerpc" => powerpc::compute_abi_info(self),
            "powerpc64" => powerpc64::compute_abi_info(cx, self),
            "s390x" => s390x::compute_abi_info(cx, self),
            "msp430" => msp430::compute_abi_info(self),
            "sparc" => sparc::compute_abi_info(cx, self),
            "sparc64" => sparc64::compute_abi_info(cx, self),
            "nvptx" => nvptx::compute_abi_info(self),
            "nvptx64" => nvptx64::compute_abi_info(self),
            "hexagon" => hexagon::compute_abi_info(self),
            "riscv32" | "riscv64" => riscv::compute_abi_info(cx, self),
            "wasm32" if cx.target_spec().target_os != "emscripten" => {
                wasm32_bindgen_compat::compute_abi_info(self)
            }
            "wasm32" | "asmjs" => wasm32::compute_abi_info(cx, self),
            a => return Err(format!("unrecognized arch \"{}\" in target specification", a)),
        }

        if let PassMode::Indirect(ref mut attrs, _) = self.ret.mode {
            attrs.set(ArgAttribute::StructRet);
        }

        Ok(())
    }
}
