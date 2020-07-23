//! A lightweight version of [pin-project] written with declarative macros.
//!
//! ## Examples
//!
//! [`pin_project!`] macro creates a projection type covering all the fields of struct.
//!
//! ```rust
//! use pin_project_lite::pin_project;
//! use std::pin::Pin;
//!
//! pin_project! {
//!     struct Struct<T, U> {
//!         #[pin]
//!         pinned: T,
//!         unpinned: U,
//!     }
//! }
//!
//! impl<T, U> Struct<T, U> {
//!     fn method(self: Pin<&mut Self>) {
//!         let this = self.project();
//!         let _: Pin<&mut T> = this.pinned; // Pinned reference to the field
//!         let _: &mut U = this.unpinned; // Normal reference to the field
//!     }
//! }
//! ```
//!
//! ## [pin-project] vs pin-project-lite
//!
//! Here are some similarities and differences compared to [pin-project].
//!
//! ### Similar: Safety
//!
//! pin-project-lite guarantees safety in much the same way as [pin-project]. Both are completely safe unless you write other unsafe code.
//!
//! ### Different: Minimal design
//!
//! This library does not tackle as expansive of a range of use cases as [pin-project] does. If your use case is not already covered, please use [pin-project].
//!
//! ### Different: No proc-macro related dependencies
//!
//! This is the **only** reason to use this crate. However, **if you already have proc-macro related dependencies in your crate's dependency graph, there is no benefit from using this crate.** (Note: There is almost no difference in the amount of code generated between [pin-project] and pin-project-lite.)
//!
//! ### Different: No useful error messages
//!
//! This macro does not handle any invalid input. So error messages are not to be useful in most cases. If you do need useful error messages, then upon error you can pass the same input to [pin-project] to receive a helpful description of the compile error.
//!
//! ### Different: Structs only
//!
//! pin-project-lite will refuse anything other than a braced struct with named fields. Enums and tuple structs are not supported.
//!
//! ### Different: No support for custom Drop implementation
//!
//! pin-project supports this by [`#[pinned_drop]`][pinned-drop].
//!
//! ### Different: No support for custom Unpin implementation
//!
//! pin-project supports this by [`UnsafeUnpin`][unsafe-unpin] and [`!Unpin`][not-unpin].
//!
//! ### Different: No support for pattern matching and destructing
//!
//! [pin-project supports this.][naming]
//!
//! [`pin_project!`]: https://docs.rs/pin-project-lite/0.1/pin_project_lite/macro.pin_project.html
//! [naming]: https://docs.rs/pin-project/0.4/pin_project/attr.pin_project.html
//! [not-unpin]: https://docs.rs/pin-project/0.4/pin_project/attr.pin_project.html#unpin
//! [pin-project]: https://github.com/taiki-e/pin-project
//! [pinned-drop]: https://docs.rs/pin-project/0.4/pin_project/attr.pin_project.html#pinned_drop
//! [unsafe-unpin]: https://docs.rs/pin-project/0.4/pin_project/attr.pin_project.html#unsafeunpin

#![no_std]
#![recursion_limit = "256"]
#![doc(html_root_url = "https://docs.rs/pin-project-lite/0.1.7")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms, single_use_lifetimes), allow(dead_code))
))]
#![warn(unsafe_code)]
#![warn(rust_2018_idioms, single_use_lifetimes, unreachable_pub)]
#![warn(clippy::all, clippy::default_trait_access)]
// mem::take and #[non_exhaustive] requires Rust 1.40
#![allow(clippy::mem_replace_with_default, clippy::manual_non_exhaustive)]

/// A macro that creates a projection type covering all the fields of struct.
///
/// This macro creates a projection type according to the following rules:
///
/// * For the field that uses `#[pin]` attribute, makes the pinned reference to the field.
/// * For the other fields, makes the unpinned reference to the field.
///
/// And the following methods are implemented on the original type:
///
/// ```rust
/// # use std::pin::Pin;
/// # type Projection<'a> = &'a ();
/// # type ProjectionRef<'a> = &'a ();
/// # trait Dox {
/// fn project(self: Pin<&mut Self>) -> Projection<'_>;
/// fn project_ref(self: Pin<&Self>) -> ProjectionRef<'_>;
/// # }
/// ```
///
/// The visibility of the projected type and projection method is based on the
/// original type. However, if the visibility of the original type is `pub`,
/// the visibility of the projected type and the projection method is `pub(crate)`.
///
/// ## Safety
///
/// `pin_project!` macro guarantees safety in much the same way as [pin-project] crate.
/// Both are completely safe unless you write other unsafe code.
///
/// See [pin-project] crate for more details.
///
/// ## Examples
///
/// ```rust
/// use pin_project_lite::pin_project;
/// use std::pin::Pin;
///
/// pin_project! {
///     struct Struct<T, U> {
///         #[pin]
///         pinned: T,
///         unpinned: U,
///     }
/// }
///
/// impl<T, U> Struct<T, U> {
///     fn method(self: Pin<&mut Self>) {
///         let this = self.project();
///         let _: Pin<&mut T> = this.pinned; // Pinned reference to the field
///         let _: &mut U = this.unpinned; // Normal reference to the field
///     }
/// }
/// ```
///
/// If you want to call the `project()` method multiple times or later use the
/// original [`Pin`] type, it needs to use [`.as_mut()`][`Pin::as_mut`] to avoid
/// consuming the [`Pin`].
///
/// If you want to ensure that [`Unpin`] is not implemented, use `#[pin]`
/// attribute for a [`PhantomPinned`] field.
///
/// ```rust
/// use pin_project_lite::pin_project;
/// use std::marker::PhantomPinned;
///
/// pin_project! {
///     struct Struct<T> {
///         field: T,
///         #[pin]
///         _pin: PhantomPinned,
///     }
/// }
/// ```
///
/// Note that using [`PhantomPinned`] without `#[pin]` attribute has no effect.
///
/// [`PhantomPinned`]: core::marker::PhantomPinned
/// [`Pin::as_mut`]: core::pin::Pin::as_mut
/// [`Pin`]: core::pin::Pin
/// [pin-project]: https://github.com/taiki-e/pin-project
#[macro_export]
macro_rules! pin_project {
    ($($tt:tt)*) => {
        $crate::__pin_project_internal! { $($tt)* }
    };
}

// limitations:
// * no support for tuple structs and enums.
// * no support for naming the projection types.
// * no support for multiple trait/lifetime bounds.
// * no support for `Self` in where clauses.
// * no support for overlapping lifetime names.
// * no interoperability with other field attributes.
// etc...

// Not public API.
#[doc(hidden)]
#[macro_export]
macro_rules! __pin_project_internal {
    // determine_visibility
    (
        $(#[$attrs:meta])*
        pub struct $ident:ident $(<
            $( $lifetime:lifetime $(: $lifetime_bound:lifetime)? ),* $(,)?
            $( $generics:ident
                $(: $generics_bound:path)?
                $(: ?$generics_unsized_bound:path)?
                $(: $generics_lifetime_bound:lifetime)?
                $(= $generics_default:ty)?
            ),* $(,)?
        >)?
        $(where
            $( $where_clause_ty:ty
                $(: $where_clause_bound:path)?
                $(: ?$where_clause_unsized_bound:path)?
                $(: $where_clause_lifetime_bound:lifetime)?
            ),* $(,)?
        )?
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
    ) => {
        $crate::__pin_project_internal! { @internal (pub(crate))
            $(#[$attrs])*
            pub struct $ident $(<
                $( $lifetime $(: $lifetime_bound)? ),*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                    $(= $generics_default)?
                ),*
            >)?
            $(where
                $( $where_clause_ty
                    $(: $where_clause_bound)?
                    $(: ?$where_clause_unsized_bound)?
                    $(: $where_clause_lifetime_bound)?
                ),*
            )?
            {
                $(
                    $(#[$pin])?
                    $field_vis $field: $field_ty
                ),+
            }
        }
    };
    (
        $(#[$attrs:meta])*
        $vis:vis struct $ident:ident $(<
            $( $lifetime:lifetime $(: $lifetime_bound:lifetime)? ),* $(,)?
            $( $generics:ident
                $(: $generics_bound:path)?
                $(: ?$generics_unsized_bound:path)?
                $(: $generics_lifetime_bound:lifetime)?
                $(= $generics_default:ty)?
            ),* $(,)?
        >)?
        $(where
            $( $where_clause_ty:ty
                $(: $where_clause_bound:path)?
                $(: ?$where_clause_unsized_bound:path)?
                $(: $where_clause_lifetime_bound:lifetime)?
            ),* $(,)?
        )?
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+ $(,)?
        }
    ) => {
        $crate::__pin_project_internal! { @internal ($vis)
            $(#[$attrs])*
            $vis struct $ident $(<
                $( $lifetime $(: $lifetime_bound)? ),*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                    $(= $generics_default)?
                ),*
            >)?
            $(where
                $( $where_clause_ty
                    $(: $where_clause_bound)?
                    $(: ?$where_clause_unsized_bound)?
                    $(: $where_clause_lifetime_bound)?
                ),*
            )?
            {
                $(
                    $(#[$pin])?
                    $field_vis $field: $field_ty
                ),+
            }
        }
    };

    (@internal ($proj_vis:vis)
        $(#[$attrs:meta])*
        $vis:vis struct $ident:ident $(<
            $( $lifetime:lifetime $(: $lifetime_bound:lifetime)? ),*
            $( $generics:ident
                $(: $generics_bound:path)?
                $(: ?$generics_unsized_bound:path)?
                $(: $generics_lifetime_bound:lifetime)?
                $(= $generics_default:ty)?
            ),*
        >)?
        $(where
            $( $where_clause_ty:ty
                $(: $where_clause_bound:path)?
                $(: ?$where_clause_unsized_bound:path)?
                $(: $where_clause_lifetime_bound:lifetime)?
            ),*
        )?
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+
        }
    ) => {
        $(#[$attrs])*
        $vis struct $ident $(<
            $( $lifetime $(: $lifetime_bound)? ,)*
            $( $generics
                $(: $generics_bound)?
                $(: ?$generics_unsized_bound)?
                $(: $generics_lifetime_bound)?
                $(= $generics_default)?
            ),*
        >)?
        $(where
            $( $where_clause_ty
                $(: $where_clause_bound)?
                $(: ?$where_clause_unsized_bound)?
                $(: $where_clause_lifetime_bound)?
            ),*
        )?
        {
            $(
                $field_vis $field: $field_ty
            ),+
        }

        #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
        #[allow(clippy::used_underscore_binding)]
        const _: () = {
            $crate::__pin_project_internal! { @make_proj_ty ($proj_vis)
                $vis struct $ident $(<
                    $( $lifetime $(: $lifetime_bound)? ),*
                    $( $generics
                        $(: $generics_bound)?
                        $(: ?$generics_unsized_bound)?
                        $(: $generics_lifetime_bound)?
                        $(= $generics_default)?
                    ),*
                >)?
                $(where
                    $( $where_clause_ty
                        $(: $where_clause_bound)?
                        $(: ?$where_clause_unsized_bound)?
                        $(: $where_clause_lifetime_bound)?
                    ),*
                )?
                {
                    $(
                        $(#[$pin])?
                        $field_vis $field: $field_ty
                    ),+
                }
            }

            impl $(<
                $( $lifetime $(: $lifetime_bound)? ,)*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                ),*
            >)?
                $ident $(< $($lifetime,)* $($generics),* >)?
            $(where
                $( $where_clause_ty
                    $(: $where_clause_bound)?
                    $(: ?$where_clause_unsized_bound)?
                    $(: $where_clause_lifetime_bound)?
                ),*
            )?
            {
                $proj_vis fn project<'__pin>(
                    self: $crate::__private::Pin<&'__pin mut Self>,
                ) -> Projection<'__pin $(, $($lifetime,)* $($generics),* )?> {
                    unsafe {
                        let this = self.get_unchecked_mut();
                        Projection {
                            $(
                                $field: $crate::__pin_project_internal!(@make_unsafe_field_proj
                                    this; $(#[$pin])? $field; mut
                                )
                            ),+
                        }
                    }
                }
                $proj_vis fn project_ref<'__pin>(
                    self: $crate::__private::Pin<&'__pin Self>,
                ) -> ProjectionRef<'__pin $(, $($lifetime,)* $($generics),* )?> {
                    unsafe {
                        let this = self.get_ref();
                        ProjectionRef {
                            $(
                                $field: $crate::__pin_project_internal!(@make_unsafe_field_proj
                                    this; $(#[$pin])? $field;
                                )
                            ),+
                        }
                    }
                }
            }

            // Automatically create the appropriate conditional `Unpin` implementation.
            //
            // Basically this is equivalent to the following code:
            // ```rust
            // impl<T, U> Unpin for Struct<T, U> where T: Unpin {}
            // ```
            //
            // However, if struct is public and there is a private type field,
            // this would cause an E0446 (private type in public interface).
            //
            // When RFC 2145 is implemented (rust-lang/rust#48054),
            // this will become a lint, rather then a hard error.
            //
            // As a workaround for this, we generate a new struct, containing all of the pinned
            // fields from our #[pin_project] type. This struct is delcared within
            // a function, which makes it impossible to be named by user code.
            // This guarnatees that it will use the default auto-trait impl for Unpin -
            // that is, it will implement Unpin iff all of its fields implement Unpin.
            // This type can be safely declared as 'public', satisfiying the privacy
            // checker without actually allowing user code to access it.
            //
            // This allows users to apply the #[pin_project] attribute to types
            // regardless of the privacy of the types of their fields.
            //
            // See also https://github.com/taiki-e/pin-project/pull/53.
            $vis struct __Origin <'__pin $(,
                $( $lifetime $(: $lifetime_bound)? ,)*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                ),*
            )?>
            $(where
                $( $where_clause_ty
                    $(: $where_clause_bound)?
                    $(: ?$where_clause_unsized_bound)?
                    $(: $where_clause_lifetime_bound)?
                ),*
            )?
            {
                __dummy_lifetime: $crate::__private::PhantomData<&'__pin ()>,
                $(
                    $field: $crate::__pin_project_internal!(@make_unpin_bound
                        $(#[$pin])? $field_ty
                    )
                ),+
            }
            impl <'__pin $(,
                $( $lifetime $(: $lifetime_bound)? ,)*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                ),*
            )?>
                $crate::__private::Unpin for $ident $(< $($lifetime,)* $($generics),* >)?
            where
                __Origin <'__pin $(, $($lifetime,)* $($generics),* )?>: $crate::__private::Unpin
                $(,
                    $( $where_clause_ty
                        $(: $where_clause_bound)?
                        $(: ?$where_clause_unsized_bound)?
                        $(: $where_clause_lifetime_bound)?
                    ),*
                )?
            {
            }

            // Ensure that struct does not implement `Drop`.
            //
            // There are two possible cases:
            // 1. The user type does not implement Drop. In this case,
            // the first blanked impl will not apply to it. This code
            // will compile, as there is only one impl of MustNotImplDrop for the user type
            // 2. The user type does impl Drop. This will make the blanket impl applicable,
            // which will then comflict with the explicit MustNotImplDrop impl below.
            // This will result in a compilation error, which is exactly what we want.
            trait MustNotImplDrop {}
            #[allow(clippy::drop_bounds)]
            impl<T: $crate::__private::Drop> MustNotImplDrop for T {}
            impl $(<
                $( $lifetime $(: $lifetime_bound)? ,)*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                ),*
            >)?
                MustNotImplDrop for $ident $(< $($lifetime,)* $($generics),* >)?
            $(where
                $( $where_clause_ty
                    $(: $where_clause_bound)?
                    $(: ?$where_clause_unsized_bound)?
                    $(: $where_clause_lifetime_bound)?
                ),*
            )?
            {
            }

            // Ensure that it's impossible to use pin projections on a #[repr(packed)] struct.
            //
            // Taking a reference to a packed field is unsafe, amd appplying
            // #[deny(safe_packed_borrows)] makes sure that doing this without
            // an 'unsafe' block (which we deliberately do not generate)
            // is a hard error.
            //
            // If the struct ends up having #[repr(packed)] applied somehow,
            // this will generate an (unfriendly) error message. Under all reasonable
            // circumstances, we'll detect the #[repr(packed)] attribute, and generate
            // a much nicer error above.
            //
            // See https://github.com/taiki-e/pin-project/pull/34 for more details.
            #[deny(safe_packed_borrows)]
            fn __assert_not_repr_packed $(<
                $( $lifetime $(: $lifetime_bound)? ,)*
                $( $generics
                    $(: $generics_bound)?
                    $(: ?$generics_unsized_bound)?
                    $(: $generics_lifetime_bound)?
                ),*
            >)?
            (
                this: &$ident $(< $($lifetime,)* $($generics),* >)?
            )
            $(where
                $( $where_clause_ty
                    $(: $where_clause_bound)?
                    $(: ?$where_clause_unsized_bound)?
                    $(: $where_clause_lifetime_bound)?
                ),*
            )?
            {
                $(
                    &this.$field;
                )+
            }
        };
    };

    // make_proj_ty
    (@make_proj_ty ($proj_vis:vis)
        $vis:vis struct $ident:ident $(<
            $( $lifetime:lifetime $(: $lifetime_bound:lifetime)? ),*
            $( $generics:ident
                $(: $generics_bound:path)?
                $(: ?$generics_unsized_bound:path)?
                $(: $generics_lifetime_bound:lifetime)?
                $(= $generics_default:ty)?
            ),*
        >)?
        where
            $( $where_clause_ty:ty
                $(: $where_clause_bound:path)?
                $(: ?$where_clause_unsized_bound:path)?
                $(: $where_clause_lifetime_bound:lifetime)?
            ),*
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+
        }
    ) => {
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
        $proj_vis struct Projection <'__pin $(,
            $( $lifetime $(: $lifetime_bound)? ,)*
            $( $generics
                $(: $generics_bound)?
                $(: ?$generics_unsized_bound)?
                $(: $generics_lifetime_bound)?
            ),*
        )?>
        where
            $ident $(< $($lifetime,)* $($generics),* >)?: '__pin,
            $( $where_clause_ty
                $(: $where_clause_bound)?
                $(: ?$where_clause_unsized_bound)?
                $(: $where_clause_lifetime_bound)?
            ),*
        {
            $(
                $field_vis $field: $crate::__pin_project_internal!(@make_proj_field
                    $(#[$pin])? $field_ty; mut
                )
            ),+
        }
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
        $proj_vis struct ProjectionRef <'__pin $(,
            $( $lifetime $(: $lifetime_bound)? ,)*
            $( $generics
                $(: $generics_bound)?
                $(: ?$generics_unsized_bound)?
                $(: $generics_lifetime_bound)?
            ),*
        )?>
        where
            $ident $(< $($lifetime,)* $($generics),* >)?: '__pin,
            $( $where_clause_ty
                $(: $where_clause_bound)?
                $(: ?$where_clause_unsized_bound)?
                $(: $where_clause_lifetime_bound)?
            ),*
        {
            $(
                $field_vis $field: $crate::__pin_project_internal!(@make_proj_field
                    $(#[$pin])? $field_ty;
                )
            ),+
        }
    };
    (@make_proj_ty ($proj_vis:vis)
        $vis:vis struct $ident:ident $(<
            $( $lifetime:lifetime $(: $lifetime_bound:lifetime)? ),*
            $( $generics:ident
                $(: $generics_bound:path)?
                $(: ?$generics_unsized_bound:path)?
                $(: $generics_lifetime_bound:lifetime)?
                $(= $generics_default:ty)?
            ),*
        >)?
        {
            $(
                $(#[$pin:ident])?
                $field_vis:vis $field:ident: $field_ty:ty
            ),+
        }
    ) => {
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
        $proj_vis struct Projection <'__pin $(,
            $( $lifetime $(: $lifetime_bound)? ,)*
            $( $generics
                $(: $generics_bound)?
                $(: ?$generics_unsized_bound)?
                $(: $generics_lifetime_bound)?
            ),*
        )?>
        where
            $ident $(< $($lifetime,)* $($generics),* >)?: '__pin,
        {
            $(
                $field_vis $field: $crate::__pin_project_internal!(@make_proj_field
                    $(#[$pin])? $field_ty; mut
                )
            ),+
        }
        #[allow(dead_code)] // This lint warns unused fields/variants.
        #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
        $proj_vis struct ProjectionRef <'__pin $(,
            $( $lifetime $(: $lifetime_bound)? ,)*
            $( $generics
                $(: $generics_bound)?
                $(: ?$generics_unsized_bound)?
                $(: $generics_lifetime_bound)?
            ),*
        )?>
        where
            $ident $(< $($lifetime,)* $($generics),* >)?: '__pin,
        {
            $(
                $field_vis $field: $crate::__pin_project_internal!(@make_proj_field
                    $(#[$pin])? $field_ty;
                )
            ),+
        }
    };

    // make_unpin_bound
    (@make_unpin_bound
        #[pin]
        $field_ty:ty
    ) => {
        $field_ty
    };
    (@make_unpin_bound
        $field_ty:ty
    ) => {
        $crate::__private::AlwaysUnpin<$field_ty>
    };

    // make_unsafe_field_proj
    (@make_unsafe_field_proj
        $this:ident;
        #[pin]
        $field:ident;
        $($mut:ident)?
    ) => {
        $crate::__private::Pin::new_unchecked(&$($mut)? $this.$field)
    };
    (@make_unsafe_field_proj
        $this:ident;
        $field:ident;
        $($mut:ident)?
    ) => {
        &$($mut)? $this.$field
    };

    // make_proj_field
    (@make_proj_field
        #[pin]
        $field_ty:ty;
        $($mut:ident)?
    ) => {
        $crate::__private::Pin<&'__pin $($mut)? ($field_ty)>
    };
    (@make_proj_field
        $field_ty:ty;
        $($mut:ident)?
    ) => {
        &'__pin $($mut)? ($field_ty)
    };

    // limitation: no useful error messages (wontfix)
}

// Not public API.
#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use core::{
        marker::{PhantomData, Unpin},
        ops::Drop,
        pin::Pin,
    };

    // This is an internal helper struct used by `pin_project!`.
    #[doc(hidden)]
    pub struct AlwaysUnpin<T: ?Sized>(PhantomData<T>);

    impl<T: ?Sized> Unpin for AlwaysUnpin<T> {}
}
