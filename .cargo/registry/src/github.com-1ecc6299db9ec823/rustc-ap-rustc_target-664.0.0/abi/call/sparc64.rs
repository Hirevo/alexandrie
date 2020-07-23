// FIXME: This needs an audit for correctness and completeness.

use crate::abi::call::{ArgAbi, FnAbi, Reg, RegKind, Uniform};
use crate::abi::{HasDataLayout, LayoutOf, TyAndLayout, TyAndLayoutMethods};

fn is_homogeneous_aggregate<'a, Ty, C>(cx: &C, arg: &mut ArgAbi<'a, Ty>) -> Option<Uniform>
where
    Ty: TyAndLayoutMethods<'a, C> + Copy,
    C: LayoutOf<Ty = Ty, TyAndLayout = TyAndLayout<'a, Ty>> + HasDataLayout,
{
    arg.layout.homogeneous_aggregate(cx).ok().and_then(|ha| ha.unit()).and_then(|unit| {
        // Ensure we have at most eight uniquely addressable members.
        if arg.layout.size > unit.size.checked_mul(8, cx).unwrap() {
            return None;
        }

        let valid_unit = match unit.kind {
            RegKind::Integer => false,
            RegKind::Float => true,
            RegKind::Vector => arg.layout.size.bits() == 128,
        };

        valid_unit.then_some(Uniform { unit, total: arg.layout.size })
    })
}

fn classify_ret<'a, Ty, C>(cx: &C, ret: &mut ArgAbi<'a, Ty>)
where
    Ty: TyAndLayoutMethods<'a, C> + Copy,
    C: LayoutOf<Ty = Ty, TyAndLayout = TyAndLayout<'a, Ty>> + HasDataLayout,
{
    if !ret.layout.is_aggregate() {
        ret.extend_integer_width_to(64);
        return;
    }

    if let Some(uniform) = is_homogeneous_aggregate(cx, ret) {
        ret.cast_to(uniform);
        return;
    }
    let size = ret.layout.size;
    let bits = size.bits();
    if bits <= 256 {
        let unit = Reg::i64();
        ret.cast_to(Uniform { unit, total: size });
        return;
    }

    // don't return aggregates in registers
    ret.make_indirect();
}

fn classify_arg<'a, Ty, C>(cx: &C, arg: &mut ArgAbi<'a, Ty>)
where
    Ty: TyAndLayoutMethods<'a, C> + Copy,
    C: LayoutOf<Ty = Ty, TyAndLayout = TyAndLayout<'a, Ty>> + HasDataLayout,
{
    if !arg.layout.is_aggregate() {
        arg.extend_integer_width_to(64);
        return;
    }

    if let Some(uniform) = is_homogeneous_aggregate(cx, arg) {
        arg.cast_to(uniform);
        return;
    }

    let total = arg.layout.size;
    if total.bits() > 128 {
        arg.make_indirect();
        return;
    }

    arg.cast_to(Uniform { unit: Reg::i64(), total });
}

pub fn compute_abi_info<'a, Ty, C>(cx: &C, fn_abi: &mut FnAbi<'a, Ty>)
where
    Ty: TyAndLayoutMethods<'a, C> + Copy,
    C: LayoutOf<Ty = Ty, TyAndLayout = TyAndLayout<'a, Ty>> + HasDataLayout,
{
    if !fn_abi.ret.is_ignore() {
        classify_ret(cx, &mut fn_abi.ret);
    }

    for arg in &mut fn_abi.args {
        if arg.is_ignore() {
            continue;
        }
        classify_arg(cx, arg);
    }
}
