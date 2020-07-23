extern crate subtle;

use subtle::*;

#[test]
#[should_panic]
fn slices_equal_different_lengths() {
    let a: [u8; 3] = [0, 0, 0];
    let b: [u8; 4] = [0, 0, 0, 0];

    assert_eq!((&a).ct_eq(&b).unwrap_u8(), 1);
}

#[test]
fn slices_equal() {
    let a: [u8; 8] = [1, 2, 3, 4, 5, 6, 7, 8];
    let b: [u8; 8] = [1, 2, 3, 4, 4, 3, 2, 1];

    let a_eq_a = (&a).ct_eq(&a);
    let a_eq_b = (&a).ct_eq(&b);

    assert_eq!(a_eq_a.unwrap_u8(), 1);
    assert_eq!(a_eq_b.unwrap_u8(), 0);

    let c: [u8; 16] = [0u8; 16];

    let a_eq_c = (&a).ct_eq(&c);
    assert_eq!(a_eq_c.unwrap_u8(), 0);
}

#[test]
fn conditional_assign_i32() {
    let mut a: i32 = 5;
    let b: i32 = 13;

    a.conditional_assign(&b, 0.into());
    assert_eq!(a, 5);
    a.conditional_assign(&b, 1.into());
    assert_eq!(a, 13);
}

#[test]
fn conditional_assign_i64() {
    let mut c: i64 = 2343249123;
    let d: i64 = 8723884895;

    c.conditional_assign(&d, 0.into());
    assert_eq!(c, 2343249123);
    c.conditional_assign(&d, 1.into());
    assert_eq!(c, 8723884895);
}

macro_rules! generate_integer_conditional_select_tests {
    ($($t:ty)*) => ($(
        let x: $t = 0;  // all 0 bits
        let y: $t = !0; // all 1 bits

        assert_eq!(<$t>::conditional_select(&x, &y, 0.into()), 0);
        assert_eq!(<$t>::conditional_select(&x, &y, 1.into()), y);
    )*)
}

#[test]
fn integer_conditional_select() {
    generate_integer_conditional_select_tests!(u8 u16 u32 u64);
    generate_integer_conditional_select_tests!(i8 i16 i32 i64);
    #[cfg(feature = "i128")]
    generate_integer_conditional_select_tests!(i128 u128);
}

#[test]
fn custom_conditional_select_i16() {
    let x: i16 = 257;
    let y: i16 = 514;

    assert_eq!(i16::conditional_select(&x, &y, 0.into()), 257);
    assert_eq!(i16::conditional_select(&x, &y, 1.into()), 514);
}

macro_rules! generate_integer_equal_tests {
    ($($t:ty),*) => ($(
        let y: $t = 0;  // all 0 bits
        let z: $t = !0; // all 1 bits

        let x = z;

        assert_eq!(x.ct_eq(&y).unwrap_u8(), 0);
        assert_eq!(x.ct_eq(&z).unwrap_u8(), 1);
    )*)
}

#[test]
fn integer_equal() {
    generate_integer_equal_tests!(u8, u16, u32, u64);
    generate_integer_equal_tests!(i8, i16, i32, i64);
    #[cfg(feature = "i128")]
    generate_integer_equal_tests!(i128, u128);
    generate_integer_equal_tests!(isize, usize);
}

#[test]
fn choice_into_bool() {
    let choice_true: bool = Choice::from(1).into();

    assert!(choice_true);

    let choice_false: bool = Choice::from(0).into();

    assert!(!choice_false);
}
