// Copyright (c) 2017 Gilad Naaman
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/// Macro to create a local `base_ptr` raw pointer of the given type, avoiding UB as
/// much as is possible currently.
#[cfg(maybe_uninit)]
#[macro_export]
#[doc(hidden)]
macro_rules! _memoffset__let_base_ptr {
    ($name:ident, $type:path) => {
        // No UB here, and the pointer does not dangle, either.
        // But we have to make sure that `uninit` lives long enough,
        // so it has to be in the same scope as `$name`. That's why
        // `let_base_ptr` declares a variable (several, actually)
        // instead of returning one.
        let uninit = $crate::mem::MaybeUninit::<$type>::uninit();
        let $name = uninit.as_ptr();
    };
}
#[cfg(not(maybe_uninit))]
#[macro_export]
#[doc(hidden)]
macro_rules! _memoffset__let_base_ptr {
    ($name:ident, $type:path) => {
        // No UB right here, but we will later dereference this pointer to
        // offset into a field, and that is UB when the pointer is dangling.
        let $name = $crate::mem::align_of::<$type>() as *const $type;
    };
}

/// Deref-coercion protection macro.
#[macro_export]
#[doc(hidden)]
macro_rules! _memoffset__field_check {
    ($type:path, $field:tt) => {
        // Make sure the field actually exists. This line ensures that a
        // compile-time error is generated if $field is accessed through a
        // Deref impl.
        #[cfg_attr(allow_clippy, allow(clippy::unneeded_field_pattern))]
        let $type { $field: _, .. };
    };
}

/// Computes a const raw pointer to the given field of the given base pointer
/// to the given parent type.
///
/// The `base` pointer *must not* be dangling, but it *may* point to
/// uninitialized memory.
#[macro_export(local_inner_macros)]
macro_rules! raw_field {
    ($base:expr, $parent:path, $field:tt) => {{
        _memoffset__field_check!($parent, $field);
        let base_ptr: *const $parent = $base;

        // Get the field address. This is UB because we are creating a reference to
        // the uninitialized field. Will be updated to use `&raw` before rustc
        // starts exploiting such UB.
        // Crucially, we know that this will not trigger a deref coercion because
        // of the `field_check!` we did above.
        #[allow(unused_unsafe)] // for when the macro is used in an unsafe block
        unsafe {
            &(*base_ptr).$field as *const _
        }
    }};
}

/// Calculates the offset of the specified field from the start of the struct.
///
/// ## Examples
/// ```
/// #[macro_use]
/// extern crate memoffset;
///
/// #[repr(C, packed)]
/// struct Foo {
///     a: u32,
///     b: u64,
///     c: [u8; 5]
/// }
///
/// fn main() {
///     assert_eq!(offset_of!(Foo, a), 0);
///     assert_eq!(offset_of!(Foo, b), 4);
/// }
/// ```
#[cfg(not(feature = "unstable_const"))]
#[macro_export(local_inner_macros)]
macro_rules! offset_of {
    ($parent:path, $field:tt) => {{
        // Get a base pointer.
        _memoffset__let_base_ptr!(base_ptr, $parent);
        // Get field pointer.
        let field_ptr = raw_field!(base_ptr, $parent, $field);
        // Compute offset.
        (field_ptr as usize) - (base_ptr as usize)
    }};
}

#[cfg(feature = "unstable_const")]
#[macro_export(local_inner_macros)]
macro_rules! offset_of {
    ($parent:path, $field:tt) => {{
        // Get a base pointer.
        // No UB here, and the pointer does not dangle, either.
        let uninit = $crate::mem::MaybeUninit::<$parent>::uninit();
        #[allow(unused_unsafe)] // for when the macro is used in an unsafe block
        unsafe {
            // This, on the other hand, *is* UB, since we're creating a reference
            // to uninitialized data.
            // Unfortunately it's the best we can do at the moment.
            let base_ptr = $crate::mem::transmute::<_, &$parent>(&uninit) as *const $parent;
            // Get a field pointer.
            let field_ptr = raw_field!(base_ptr, $parent, $field);
            // Compute offset.
            (field_ptr as *const u8).offset_from(base_ptr as *const u8) as usize
        }
    }};
}

#[cfg(test)]
mod tests {
    #[test]
    fn offset_simple() {
        #[repr(C)]
        struct Foo {
            a: u32,
            b: [u8; 2],
            c: i64,
        }

        assert_eq!(offset_of!(Foo, a), 0);
        assert_eq!(offset_of!(Foo, b), 4);
        assert_eq!(offset_of!(Foo, c), 8);
    }

    #[test]
    #[cfg_attr(miri, ignore)] // this creates unaligned references
    fn offset_simple_packed() {
        #[repr(C, packed)]
        struct Foo {
            a: u32,
            b: [u8; 2],
            c: i64,
        }

        assert_eq!(offset_of!(Foo, a), 0);
        assert_eq!(offset_of!(Foo, b), 4);
        assert_eq!(offset_of!(Foo, c), 6);
    }

    #[test]
    fn tuple_struct() {
        #[repr(C)]
        struct Tup(i32, i32);

        assert_eq!(offset_of!(Tup, 0), 0);
        assert_eq!(offset_of!(Tup, 1), 4);
    }

    #[test]
    fn path() {
        mod sub {
            #[repr(C)]
            pub struct Foo {
                pub x: u32,
            }
        }

        assert_eq!(offset_of!(sub::Foo, x), 0);
    }

    #[test]
    fn inside_generic_method() {
        struct Pair<T, U>(T, U);

        fn foo<T, U>(_: Pair<T, U>) -> usize {
            offset_of!(Pair<T, U>, 1)
        }

        assert_eq!(foo(Pair(0, 0)), 4);
    }

    #[test]
    fn test_raw_field() {
        #[repr(C)]
        struct Foo {
            a: u32,
            b: [u8; 2],
            c: i64,
        }

        let f: Foo = Foo {
            a: 0,
            b: [0, 0],
            c: 0,
        };
        let f_ptr = &f as *const _;
        assert_eq!(f_ptr as usize + 0, raw_field!(f_ptr, Foo, a) as usize);
        assert_eq!(f_ptr as usize + 4, raw_field!(f_ptr, Foo, b) as usize);
        assert_eq!(f_ptr as usize + 8, raw_field!(f_ptr, Foo, c) as usize);
    }

    #[cfg(feature = "unstable_const")]
    #[test]
    fn const_offset() {
        #[repr(C)]
        struct Foo {
            a: u32,
            b: [u8; 2],
            c: i64,
        }

        assert_eq!([0; offset_of!(Foo, b)].len(), 4);
    }
}
