//! Various data structures used by the Rust compiler. The intention
//! is that code in here should be not be *specific* to rustc, so that
//! it can be easily unit tested and so forth.
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_root_url = "https://doc.rust-lang.org/nightly/")]
#![feature(rustc_private, in_band_lifetimes)]
#![feature(unboxed_closures)]
#![feature(generators)]
#![feature(generator_trait)]
#![feature(fn_traits)]
#![feature(min_specialization)]
#![feature(optin_builtin_traits)]
#![feature(nll)]
#![feature(allow_internal_unstable)]
#![feature(hash_raw_entry)]
#![feature(stmt_expr_attributes)]
#![feature(core_intrinsics)]
#![feature(test)]
#![feature(associated_type_bounds)]
#![feature(thread_id_value)]
#![feature(extend_one)]
#![allow(rustc::default_hash_types)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

#[inline(never)]
#[cold]
pub fn cold_path<F: FnOnce() -> R, R>(f: F) -> R {
    f()
}

#[macro_export]
macro_rules! likely {
    ($e:expr) => {
        #[allow(unused_unsafe)]
        {
            unsafe { std::intrinsics::likely($e) }
        }
    };
}

#[macro_export]
macro_rules! unlikely {
    ($e:expr) => {
        #[allow(unused_unsafe)]
        {
            unsafe { std::intrinsics::unlikely($e) }
        }
    };
}

pub mod base_n;
pub mod binary_search_util;
pub mod box_region;
pub mod captures;
pub mod const_cstr;
pub mod flock;
pub mod fx;
pub mod graph;
pub mod jobserver;
pub mod macros;
pub mod map_in_place;
pub mod obligation_forest;
pub mod owning_ref;
pub mod ptr_key;
pub mod sip128;
pub mod small_c_str;
pub mod snapshot_map;
pub mod stable_map;
pub mod svh;
pub use ena::snapshot_vec;
pub mod sorted_map;
pub mod stable_set;
#[macro_use]
pub mod stable_hasher;
pub mod sharded;
pub mod stack;
pub mod sync;
pub mod thin_vec;
pub mod tiny_list;
pub mod transitive_relation;
pub use ena::undo_log;
pub use ena::unify;
mod atomic_ref;
pub mod fingerprint;
pub mod profiling;
pub mod vec_linked_list;
pub mod work_queue;
pub use atomic_ref::AtomicRef;
pub mod frozen;

pub struct OnDrop<F: Fn()>(pub F);

impl<F: Fn()> OnDrop<F> {
    /// Forgets the function which prevents it from running.
    /// Ensure that the function owns no memory, otherwise it will be leaked.
    #[inline]
    pub fn disable(self) {
        std::mem::forget(self);
    }
}

impl<F: Fn()> Drop for OnDrop<F> {
    #[inline]
    fn drop(&mut self) {
        (self.0)();
    }
}

// See comments in src/librustc_middle/lib.rs
#[doc(hidden)]
pub fn __noop_fix_for_27438() {}
