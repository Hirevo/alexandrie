// Reference: PTX Writer's Guide to Interoperability
// http://docs.nvidia.com/cuda/ptx-writers-guide-to-interoperability

use crate::abi::call::{ArgAbi, FnAbi};

fn classify_ret<Ty>(ret: &mut ArgAbi<'_, Ty>) {
    if ret.layout.is_aggregate() && ret.layout.size.bits() > 64 {
        ret.make_indirect();
    } else {
        ret.extend_integer_width_to(64);
    }
}

fn classify_arg<Ty>(arg: &mut ArgAbi<'_, Ty>) {
    if arg.layout.is_aggregate() && arg.layout.size.bits() > 64 {
        arg.make_indirect();
    } else {
        arg.extend_integer_width_to(64);
    }
}

pub fn compute_abi_info<Ty>(fn_abi: &mut FnAbi<'_, Ty>) {
    if !fn_abi.ret.is_ignore() {
        classify_ret(&mut fn_abi.ret);
    }

    for arg in &mut fn_abi.args {
        if arg.is_ignore() {
            continue;
        }
        classify_arg(arg);
    }
}
