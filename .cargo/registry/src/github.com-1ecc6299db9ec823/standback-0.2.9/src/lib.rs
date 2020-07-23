#![allow(non_camel_case_types, unstable_name_collisions)]
#![cfg_attr(not(std), no_std)]

//! Standback backports a number of methods, structs, and macros that have been
//! stabilized in the Rust standard library since 1.31.0. This allows crate
//! authors to depend on Standback rather than forcing downstream users to
//! upgrade their compiler (or not use the new feature at all).
//!
//! Due to a variety of restrictions in the Rust, it is not possible to
//! implement everything that has been stabilized.
//!
//! # Usage
//!
//! If you are using methods on already-existing structs, you should use the
//! following:
//!
//! ```rust,no_run
//! use standback::prelude::*;
//! ```
//!
//! Additionally, if you are using newly stabilized structs, types, or anything
//! else that would normally have to be imported, use `standback` instead of
//! `std`:
//!
//! ```rust,no_run
//! use standback::mem::take;
//! ```
//!
//! It is _highly_ recommended to use `#![allow(unstable_name_collisions)]`, as
//! that's the whole point of this crate. Just be extra-careful to not do it for
//! anything that _can't_ be backported.
//!
//! # `#![no_std]` support
//!
//! By default, there standard library is used where necessary. If support for
//! `#![no_std]` is required, use `default-features = false`.
//!
//! An allocator is not required for any backported item. If any require an
//! allocator in the future, it will be gated under an `alloc` feature.
//!
//! # Methods on existing structs
//!
//! The following methods and constants are available via the prelude:
//!
//! ```rust,ignore
//! // 1.44
//! PathBuf::with_capacity
//! PathBuf::capacity
//! PathBuf::clear
//! PathBuf::reserve
//! PathBuf::reserve_exact
//! PathBuf::shrink_to_fit
//! Layout::align_to
//! Layout::pad_to_align
//! Layout::array
//! Layout::extend
//!
//! // 1.43
//! f32::RADIX
//! f32::MANTISSA_DIGITS
//! f32::DIGITS
//! f32::EPSILON
//! f32::MIN
//! f32::MIN_POSITIVE
//! f32::MAX
//! f32::MIN_EXP
//! f32::MAX_EXP
//! f32::MIN_10_EXP
//! f32::MAX_10_EXP
//! f32::NAN
//! f32::INFINITY
//! f32::NEG_INFINITY
//! f64::RADIX
//! f64::MANTISSA_DIGITS
//! f64::DIGITS
//! f64::EPSILON
//! f64::MIN
//! f64::MIN_POSITIVE
//! f64::MAX
//! f64::MIN_EXP
//! f64::MAX_EXP
//! f64::MIN_10_EXP
//! f64::MAX_10_EXP
//! f64::NAN
//! f64::INFINITY
//! f64::NEG_INFINITY
//! u8::MIN
//! u8::MAX
//! u16::MIN
//! u16::MAX
//! u32::MIN
//! u32::MAX
//! u64::MIN
//! u64::MAX
//! u128::MIN
//! u128::MAX
//! usize::MIN
//! usize::MAX
//! i8::MIN
//! i8::MAX
//! i16::MIN
//! i16::MAX
//! i32::MIN
//! i32::MAX
//! i64::MIN
//! i64::MAX
//! i128::MIN
//! i128::MAX
//! isize::MIN
//! isize::MAX
//!
//! // 1.42
//! CondVar::wait_while
//! CondVar::wait_timeout_while
//! ManuallyDrop::take
//!
//! // 1.41
//! Result::map_or
//! Result::map_or_else
//!
//! // 1.40
//! Option::as_deref
//! Option::as_deref_mut
//! f32::to_be_bytes
//! f32::to_le_bytes
//! f32::to_ne_bytes
//! f64::to_be_bytes
//! f64::to_le_bytes
//! f64::to_ne_bytes
//! f32::from_be_bytes
//! f32::from_le_bytes
//! f32::from_ne_bytes
//! f64::from_be_bytes
//! f64::from_le_bytes
//! f64::from_ne_bytes
//! slice::repeat
//!
//! // 1.39
//! // None :(
//!
//! // 1.38
//! <*const T>::cast
//! <*mut T>::cast
//! Duration::as_secs_f32
//! Duration::as_secs_f64
//! Duration::div_f32
//! Duration::div_f64
//! Duration::from_secs_f32
//! Duration::from_secs_f64
//! Duration::mul_f32
//! Duration::mul_f64
//! i8::rem_euclid
//! i8::checked_rem_euclid
//! i8::wrapping_rem_euclid
//! i8::overflowing_rem_euclid
//! i8::div_euclid
//! i8::checked_div_euclid
//! i8::wrapping_div_euclid
//! i8::overflowing_div_euclid
//! i16::rem_euclid
//! i16::checked_rem_euclid
//! i16::wrapping_rem_euclid
//! i16::overflowing_rem_euclid
//! i16::div_euclid
//! i16::checked_div_euclid
//! i16::wrapping_div_euclid
//! i16::overflowing_div_euclid
//! i32::rem_euclid
//! i32::checked_rem_euclid
//! i32::wrapping_rem_euclid
//! i32::overflowing_rem_euclid
//! i32::div_euclid
//! i32::checked_div_euclid
//! i32::wrapping_div_euclid
//! i32::overflowing_div_euclid
//! i64::rem_euclid
//! i64::checked_rem_euclid
//! i64::wrapping_rem_euclid
//! i64::overflowing_rem_euclid
//! i64::div_euclid
//! i64::checked_div_euclid
//! i64::wrapping_div_euclid
//! i64::overflowing_div_euclid
//! i128::rem_euclid
//! i128::checked_rem_euclid
//! i128::wrapping_rem_euclid
//! i128::overflowing_rem_euclid
//! i128::div_euclid
//! i128::checked_div_euclid
//! i128::wrapping_div_euclid
//! i128::overflowing_div_euclid
//! isize::rem_euclid
//! isize::checked_rem_euclid
//! isize::wrapping_rem_euclid
//! isize::overflowing_rem_euclid
//! isize::div_euclid
//! isize::checked_div_euclid
//! isize::wrapping_div_euclid
//! isize::overflowing_div_euclid
//! u8::rem_euclid
//! u8::checked_rem_euclid
//! u8::wrapping_rem_euclid
//! u8::overflowing_rem_euclid
//! u8::div_euclid
//! u8::checked_div_euclid
//! u8::wrapping_div_euclid
//! u8::overflowing_div_euclid
//! u16::rem_euclid
//! u16::checked_rem_euclid
//! u16::wrapping_rem_euclid
//! u16::overflowing_rem_euclid
//! u16::div_euclid
//! u16::checked_div_euclid
//! u16::wrapping_div_euclid
//! u16::overflowing_div_euclid
//! u32::rem_euclid
//! u32::checked_rem_euclid
//! u32::wrapping_rem_euclid
//! u32::overflowing_rem_euclid
//! u32::div_euclid
//! u32::checked_div_euclid
//! u32::wrapping_div_euclid
//! u32::overflowing_div_euclid
//! u64::rem_euclid
//! u64::checked_rem_euclid
//! u64::wrapping_rem_euclid
//! u64::overflowing_rem_euclid
//! u64::div_euclid
//! u64::checked_div_euclid
//! u64::wrapping_div_euclid
//! u64::overflowing_div_euclid
//! u128::rem_euclid
//! u128::checked_rem_euclid
//! u128::wrapping_rem_euclid
//! u128::overflowing_rem_euclid
//! u128::div_euclid
//! u128::checked_div_euclid
//! u128::wrapping_div_euclid
//! u128::overflowing_div_euclid
//! usize::rem_euclid
//! usize::checked_rem_euclid
//! usize::wrapping_rem_euclid
//! usize::overflowing_rem_euclid
//! usize::div_euclid
//! usize::checked_div_euclid
//! usize::wrapping_div_euclid
//! usize::overflowing_div_euclid
//! f32::rem_euclid
//! f32::div_euclid
//! f64::rem_euclid
//! f64::div_euclid
//!
//! // 1.37
//! Cell::from_mut
//! Cell<[T]>::as_slice_of_cells
//! DoubleEndedIterator::nth_back
//! Option::xor
//! slice::copy_within
//!
//! // 1.36
//! Iterator::copied
//! mem::MaybeUninit
//! task::Context
//! task::RawWaker
//! task::RawWakerVTable
//! task::Waker
//! task::Poll
//!
//! // 1.35
//! RefCell::replace_with
//! ptr::hash
//! Range::contains
//! RangeFrom::contains
//! RangeTo::contains
//! RangeInclusive::contains
//! RangeToInclusive::contains
//! Option::copied
//!
//! // 1.34
//! slice::sort_by_cached_key
//! i8::checked_pow
//! i8::saturating_pow
//! i8::wrapping_pow
//! i8::overflowing_pow
//! i16::checked_pow
//! i16::saturating_pow
//! i16::wrapping_pow
//! i16::overflowing_pow
//! i32::checked_pow
//! i32::saturating_pow
//! i32::wrapping_pow
//! i32::overflowing_pow
//! i64::checked_pow
//! i64::saturating_pow
//! i64::wrapping_pow
//! i64::overflowing_pow
//! i128::checked_pow
//! i128::saturating_pow
//! i128::wrapping_pow
//! i128::overflowing_pow
//! isize::checked_pow
//! isize::saturating_pow
//! isize::wrapping_pow
//! isize::overflowing_pow
//! u8::checked_pow
//! u8::saturating_pow
//! u8::wrapping_pow
//! u8::overflowing_pow
//! u16::checked_pow
//! u16::saturating_pow
//! u16::wrapping_pow
//! u16::overflowing_pow
//! u32::checked_pow
//! u32::saturating_pow
//! u32::wrapping_pow
//! u32::overflowing_pow
//! u64::checked_pow
//! u64::saturating_pow
//! u64::wrapping_pow
//! u64::overflowing_pow
//! u128::checked_pow
//! u128::saturating_pow
//! u128::wrapping_pow
//! u128::overflowing_pow
//! usize::checked_pow
//! usize::saturating_pow
//! usize::wrapping_pow
//! usize::overflowing_pow
//!
//! // 1.33
//! os::unix::fs::FileExt::read_exact_at
//! os::unix::fs::FileExt::write_all_at
//! Option::transpose
//! Result::transpose
//! VecDeque::resize_with
//! Duration::as_millis
//! Duration::as_micros
//! Duration::as_nanos
//!
//! // 1.32
//! i8::to_be_bytes
//! i8::to_le_bytes
//! i8::to_ne_bytes
//! i8::from_be_bytes
//! i8::from_le_bytes
//! i8::from_ne_bytes
//! i16::to_be_bytes
//! i16::to_le_bytes
//! i16::to_ne_bytes
//! i16::from_be_bytes
//! i16::from_le_bytes
//! i16::from_ne_bytes
//! i32::to_be_bytes
//! i32::to_le_bytes
//! i32::to_ne_bytes
//! i32::from_be_bytes
//! i32::from_le_bytes
//! i32::from_ne_bytes
//! i64::to_be_bytes
//! i64::to_le_bytes
//! i64::to_ne_bytes
//! i64::from_be_bytes
//! i64::from_le_bytes
//! i64::from_ne_bytes
//! i128::to_be_bytes
//! i128::to_le_bytes
//! i128::to_ne_bytes
//! i128::from_be_bytes
//! i128::from_le_bytes
//! i128::from_ne_bytes
//! isize::to_be_bytes
//! isize::to_le_bytes
//! isize::to_ne_bytes
//! isize::from_be_bytes
//! isize::from_le_bytes
//! isize::from_ne_bytes
//! u8::to_be_bytes
//! u8::to_le_bytes
//! u8::to_ne_bytes
//! u8::from_be_bytes
//! u8::from_le_bytes
//! u8::from_ne_bytes
//! u16::to_be_bytes
//! u16::to_le_bytes
//! u16::to_ne_bytes
//! u16::from_be_bytes
//! u16::from_le_bytes
//! u16::from_ne_bytes
//! u32::to_be_bytes
//! u32::to_le_bytes
//! u32::to_ne_bytes
//! u32::from_be_bytes
//! u32::from_le_bytes
//! u32::from_ne_bytes
//! u64::to_be_bytes
//! u64::to_le_bytes
//! u64::to_ne_bytes
//! u64::from_be_bytes
//! u64::from_le_bytes
//! u64::from_ne_bytes
//! u128::to_be_bytes
//! u128::to_le_bytes
//! u128::to_ne_bytes
//! u128::from_be_bytes
//! u128::from_le_bytes
//! u128::from_ne_bytes
//! usize::to_be_bytes
//! usize::to_le_bytes
//! usize::to_ne_bytes
//! usize::from_be_bytes
//! usize::from_le_bytes
//! usize::from_ne_bytes
//! ```
//!
//! # Other APIs implemented
//!
//! ```rust,ignore
//! primitive // 1.43 — requires rustc 1.32.0
//! f32::LOG10_2 // 1.43
//! f32::LOG2_10 // 1.43
//! f64::LOG10_2 // 1.43
//! f64::LOG2_10 // 1.43
//! iter::once_with // 1.43
//! mem::take // 1.40
//! iterator::Copied // 1.36
//! array::TryFromSliceError // 1.36
//! iter::from_fn // 1.34
//! iter::successors // 1.34
//! convert::TryFrom // 1.34
//! convert::TryInto // 1.34
//! num::TryFromIntError // 1.34
//! convert::identity // 1.33
//! pin::Pin // 1.33
//! marker::Unpin // 1.33
//! ```
//!
//! # Macros
//!
//! Macros should not be imported directly, but rather through the prelude.
//!
//! ```rust,ignore
//! todo! // 1.39
//! matches! // 1.42
//! ```

#![deny(rust_2018_idioms, unused_qualifications)]

// A few traits to make sealing other traits simpler.
pub trait Sealed<T: ?Sized> {}
impl<T: ?Sized> Sealed<T> for T {}

pub trait Integer: Sized {}
impl Integer for i8 {}
impl Integer for i16 {}
impl Integer for i32 {}
impl Integer for i64 {}
impl Integer for i128 {}
impl Integer for isize {}
impl Integer for u8 {}
impl Integer for u16 {}
impl Integer for u32 {}
impl Integer for u64 {}
impl Integer for u128 {}
impl Integer for usize {}

pub trait Float {}
impl Float for f32 {}
impl Float for f64 {}

#[cfg(before_1_32)]
mod v1_32;
#[cfg(before_1_33)]
mod v1_33;
#[cfg(before_1_34)]
mod v1_34;
#[cfg(before_1_35)]
mod v1_35;
#[cfg(before_1_36)]
mod v1_36;
#[cfg(before_1_37)]
mod v1_37;
#[cfg(before_1_38)]
mod v1_38;
#[cfg(before_1_39)]
mod v1_39;
#[cfg(before_1_40)]
mod v1_40;
#[cfg(before_1_41)]
mod v1_41;
#[cfg(before_1_42)]
mod v1_42;
#[cfg(before_1_43)]
mod v1_43;
#[cfg(before_1_44)]
mod v1_44;

pub mod prelude {
    #[cfg(before_1_42)]
    pub use crate::matches;
    #[cfg(before_1_32)]
    pub use crate::v1_32::{
        i128_v1_32, i16_v1_32, i32_v1_32, i64_v1_32, i8_v1_32, isize_v1_32, u128_v1_32, u16_v1_32,
        u32_v1_32, u64_v1_32, u8_v1_32, usize_v1_32,
    };
    #[cfg(all(std, before_1_33, target_family = "unix"))]
    pub use crate::v1_33::UnixFileExt_v1_33;
    #[cfg(all(std, before_1_33))]
    pub use crate::v1_33::VecDeque_v1_33;
    #[cfg(before_1_33)]
    pub use crate::v1_33::{Duration_v1_33, Option_v1_33, Result_v1_33};
    #[cfg(before_1_34)]
    pub use crate::v1_34::{Pow_v1_34, Slice_v1_34};
    #[cfg(before_1_35)]
    pub use crate::v1_35::{Option_v1_35, RangeBounds_v1_35, RefCell_v1_35};
    #[cfg(before_1_36)]
    pub use crate::v1_36::{str_v1_36, Iterator_v1_36};
    #[cfg(before_1_37)]
    pub use crate::v1_37::{
        Cell_v1_37, Cell_v1_37_, DoubleEndedIterator_v1_37, Option_v1_37, Slice_v1_37,
    };
    #[cfg(before_1_38)]
    pub use crate::v1_38::{
        ConstPtr_v1_38, Duration_v1_38, EuclidFloat_v1_38, Euclid_v1_38, MutPtr_v1_38,
    };
    #[cfg(all(std, before_1_40))]
    pub use crate::v1_40::slice_v1_40;
    #[cfg(before_1_40)]
    pub use crate::v1_40::{f32_v1_40, f64_v1_40, Option_v1_40, Option_v1_40_};
    #[cfg(before_1_41)]
    pub use crate::v1_41::Result_v1_41;
    #[cfg(all(before_1_42, std))]
    pub use crate::v1_42::Condvar_v1_42;
    #[cfg(before_1_42)]
    pub use crate::v1_42::ManuallyDrop_v1_42;
    #[cfg(before_1_43)]
    pub use crate::v1_43::{float_v1_43, int_v1_43};
    #[cfg(before_1_44)]
    pub use crate::v1_44::Layout_v1_44;
    #[cfg(all(before_1_44, std))]
    pub use crate::v1_44::PathBuf_v1_44;
    #[cfg(before_1_39)]
    pub use core::unimplemented as todo;
}

pub mod mem {
    #[cfg(before_1_40)]
    pub use crate::v1_40::take;
    #[cfg(since_1_40)]
    pub use core::mem::take;

    #[cfg(before_1_36)]
    pub use crate::v1_36::MaybeUninit;
    #[cfg(since_1_36)]
    pub use core::mem::MaybeUninit;
}
pub mod convert {
    #[cfg(before_1_33)]
    pub use crate::v1_33::identity;
    #[cfg(since_1_33)]
    pub use core::convert::identity;

    #[cfg(before_1_34)]
    pub use crate::v1_34::Infallible;
    #[cfg(since_1_34)]
    pub use core::convert::Infallible;

    #[cfg(before_1_34)]
    pub use crate::v1_34::{TryFrom, TryInto};
    #[cfg(since_1_34)]
    pub use core::convert::{TryFrom, TryInto};
}
pub mod num {
    #[cfg(before_1_34)]
    pub use crate::v1_34::TryFromIntError;
    #[cfg(since_1_34)]
    pub use core::num::TryFromIntError;
}
pub mod iter {
    #[cfg(before_1_36)]
    pub use crate::v1_36::Copied;
    #[cfg(since_1_36)]
    pub use core::iter::Copied;

    #[cfg(before_1_34)]
    pub use crate::v1_34::{from_fn, successors};
    #[cfg(since_1_34)]
    pub use core::iter::{from_fn, successors};

    #[cfg(before_1_43)]
    pub use crate::v1_43::{once_with, OnceWith};
    #[cfg(since_1_43)]
    pub use core::iter::{once_with, OnceWith};
}
pub mod marker {
    #[cfg(before_1_33)]
    pub use crate::v1_33::Unpin;
    #[cfg(since_1_33)]
    pub use core::marker::Unpin;
}
pub mod pin {
    #[cfg(before_1_33)]
    pub use crate::v1_33::Pin;
    #[cfg(since_1_33)]
    pub use core::pin::Pin;
}
pub mod task {
    #[cfg(before_1_36)]
    pub use crate::v1_36::{Context, Poll, RawWaker, RawWakerVTable, Waker};
    #[cfg(since_1_36)]
    pub use core::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};
}
pub mod ptr {
    #[cfg(before_1_35)]
    pub use crate::v1_35::hash;
    #[cfg(since_1_35)]
    pub use core::ptr::hash;
}
pub mod array {
    #[cfg(before_1_36)]
    pub use crate::v1_36::TryFromSliceError;
    #[cfg(since_1_36)]
    pub use core::array::TryFromSliceError;
}
pub mod f32 {
    pub mod consts {
        #[cfg(before_1_43)]
        pub use crate::v1_43::f32::{LOG10_2, LOG2_10};
        #[cfg(since_1_43)]
        pub use core::f32::consts::{LOG10_2, LOG2_10};
    }
}
pub mod f64 {
    pub mod consts {
        #[cfg(before_1_43)]
        pub use crate::v1_43::f64::{LOG10_2, LOG2_10};
        #[cfg(since_1_43)]
        pub use core::f64::consts::{LOG10_2, LOG2_10};
    }
}
