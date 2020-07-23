#![no_std]
#![warn(rust_2018_idioms, single_use_lifetimes)]
#![allow(dead_code)]

use core::{marker::PhantomPinned, pin::Pin};
use pin_project_lite::pin_project;

#[test]
fn projection() {
    pin_project! {
        struct Struct<T, U> {
            #[pin]
            field1: T,
            field2: U,
        }
    }

    let mut s = Struct { field1: 1, field2: 2 };
    let mut s_orig = Pin::new(&mut s);
    let s = s_orig.as_mut().project();

    let x: Pin<&mut i32> = s.field1;
    assert_eq!(*x, 1);

    let y: &mut i32 = s.field2;
    assert_eq!(*y, 2);

    assert_eq!(s_orig.as_ref().field1, 1);
    assert_eq!(s_orig.as_ref().field2, 2);

    let mut s = Struct { field1: 1, field2: 2 };

    let s = Pin::new(&mut s).project();

    let _: Pin<&mut i32> = s.field1;
    let _: &mut i32 = s.field2;
}

#[test]
fn where_clause() {
    pin_project! {
        struct Struct<T>
        where
            T: Copy,
        {
            field: T,
        }
    }
}

#[test]
fn where_clause_and_associated_type_field() {
    pin_project! {
        struct Struct1<I>
        where
            I: Iterator,
        {
            #[pin]
            field1: I,
            field2: I::Item,
        }
    }

    pin_project! {
        struct Struct2<I, J>
        where
            I: Iterator<Item = J>,
        {
            #[pin]
            field1: I,
            field2: J,
        }
    }

    pin_project! {
        pub struct Struct3<T>
        where
            T: 'static,
        {
            field: T,
        }
    }

    trait Static: 'static {}

    impl<T> Static for Struct3<T> {}
}

#[test]
fn derive_copy() {
    pin_project! {
        #[derive(Clone, Copy)]
        struct Struct<T> {
            val: T,
        }
    }

    fn is_copy<T: Copy>() {}

    is_copy::<Struct<u8>>();
}

#[test]
fn move_out() {
    struct NotCopy;

    pin_project! {
        struct Struct {
            val: NotCopy,
        }
    }

    let x = Struct { val: NotCopy };
    let _val: NotCopy = x.val;
}

#[test]
fn trait_bounds_on_type_generics() {
    pin_project! {
        pub struct Struct1<'a, T: ?Sized> {
            field: &'a mut T,
        }
    }

    pin_project! {
        pub struct Struct2<'a, T: ::core::fmt::Debug> {
            field: &'a mut T,
        }
    }

    pin_project! {
        pub struct Struct3<'a, T: core::fmt::Debug> {
            field: &'a mut T,
        }
    }

    // pin_project! {
    //     pub struct Struct4<'a, T: core::fmt::Debug + core::fmt::Display> {
    //         field: &'a mut T,
    //     }
    // }

    // pin_project! {
    //     pub struct Struct5<'a, T: core::fmt::Debug + ?Sized> {
    //         field: &'a mut T,
    //     }
    // }

    pin_project! {
        pub struct Struct6<'a, T: core::fmt::Debug = [u8; 16]> {
            field: &'a mut T,
        }
    }

    let _: Struct6<'_> = Struct6 { field: &mut [0u8; 16] };

    pin_project! {
        pub struct Struct7<T: 'static> {
            field: T,
        }
    }

    trait Static: 'static {}

    impl<T> Static for Struct7<T> {}

    pin_project! {
        pub struct Struct8<'a, 'b: 'a> {
            field1: &'a u8,
            field2: &'b u8,
        }
    }
}

#[test]
fn private_type_in_public_type() {
    pin_project! {
        pub struct PublicStruct<T> {
            #[pin]
            inner: PrivateStruct<T>,
        }
    }

    struct PrivateStruct<T>(T);
}

#[test]
fn lifetime_project() {
    pin_project! {
        struct Struct1<T, U> {
            #[pin]
            pinned: T,
            unpinned: U,
        }
    }

    pin_project! {
        struct Struct2<'a, T, U> {
            #[pin]
            pinned: &'a mut T,
            unpinned: U,
        }
    }

    impl<T, U> Struct1<T, U> {
        fn get_pin_ref<'a>(self: Pin<&'a Self>) -> Pin<&'a T> {
            self.project_ref().pinned
        }
        fn get_pin_mut<'a>(self: Pin<&'a mut Self>) -> Pin<&'a mut T> {
            self.project().pinned
        }
    }

    impl<'b, T, U> Struct2<'b, T, U> {
        fn get_pin_ref<'a>(self: Pin<&'a Self>) -> Pin<&'a &'b mut T> {
            self.project_ref().pinned
        }
        fn get_pin_mut<'a>(self: Pin<&'a mut Self>) -> Pin<&'a mut &'b mut T> {
            self.project().pinned
        }
    }
}

#[test]
fn lifetime_project_elided() {
    pin_project! {
        struct Struct1<T, U> {
            #[pin]
            pinned: T,
            unpinned: U,
        }
    }

    pin_project! {
        struct Struct2<'a, T, U> {
            #[pin]
            pinned: &'a mut T,
            unpinned: U,
        }
    }

    impl<T, U> Struct1<T, U> {
        fn get_pin_ref(self: Pin<&Self>) -> Pin<&T> {
            self.project_ref().pinned
        }
        fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut T> {
            self.project().pinned
        }
    }

    impl<'b, T, U> Struct2<'b, T, U> {
        fn get_pin_ref(self: Pin<&Self>) -> Pin<&&'b mut T> {
            self.project_ref().pinned
        }
        fn get_pin_mut(self: Pin<&mut Self>) -> Pin<&mut &'b mut T> {
            self.project().pinned
        }
    }
}

mod visibility {
    use pin_project_lite::pin_project;

    pin_project! {
        pub(crate) struct A {
            pub b: u8,
        }
    }
}

#[test]
fn visibility() {
    let mut x = visibility::A { b: 0 };
    let x = Pin::new(&mut x);
    let y = x.as_ref().project_ref();
    let _: &u8 = y.b;
    let y = x.project();
    let _: &mut u8 = y.b;
}

#[test]
fn trivial_bounds() {
    pin_project! {
        pub struct NoGenerics {
            #[pin]
            field: PhantomPinned,
        }
    }
}

#[test]
fn dst() {
    pin_project! {
        pub struct Struct1<T: ?Sized> {
            x: T,
        }
    }

    let mut x = Struct1 { x: 0_u8 };
    let x: Pin<&mut Struct1<dyn core::fmt::Debug>> = Pin::new(&mut x as _);
    let _y: &mut (dyn core::fmt::Debug) = x.project().x;

    pin_project! {
        pub struct Struct2<T: ?Sized> {
            #[pin]
            x: T,
        }
    }

    let mut x = Struct2 { x: 0_u8 };
    let x: Pin<&mut Struct2<dyn core::fmt::Debug + Unpin>> = Pin::new(&mut x as _);
    let _y: Pin<&mut (dyn core::fmt::Debug + Unpin)> = x.project().x;
}

#[allow(explicit_outlives_requirements)] // https://github.com/rust-lang/rust/issues/60993
#[test]
fn unsized_in_where_clause() {
    pin_project! {
        struct Struct3<T>
        where
            T: ?Sized,
        {
            x: T,
        }
    }

    pin_project! {
        struct Struct4<T>
        where
            T: ?Sized,
        {
            #[pin]
            x: T,
        }
    }
}

#[test]
fn dyn_type() {
    pin_project! {
        struct Struct1 {
            f: dyn core::fmt::Debug,
        }
    }

    pin_project! {
        struct Struct2 {
            #[pin]
            f: dyn core::fmt::Debug,
        }
    }

    pin_project! {
        struct Struct3 {
            f: dyn core::fmt::Debug + Send,
        }
    }

    pin_project! {
        struct Struct4 {
            #[pin]
            f: dyn core::fmt::Debug + Send,
        }
    }
}

#[test]
fn no_infer_outlives() {
    trait Bar<X> {
        type Y;
    }

    struct Example<A>(A);

    impl<X, T> Bar<X> for Example<T> {
        type Y = Option<T>;
    }

    pin_project! {
        struct Foo<A, B> {
            _x: <Example<A> as Bar<B>>::Y,
        }
    }
}
