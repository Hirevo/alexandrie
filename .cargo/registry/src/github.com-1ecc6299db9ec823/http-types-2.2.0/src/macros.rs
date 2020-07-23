/// Return early with an error.
#[macro_export]
macro_rules! bail {
    ($msg:literal $(,)?) => {
        return $crate::private::Err($crate::format_err!($msg));
    };
    ($err:expr $(,)?) => {
        return $crate::private::Err($crate::format_err!($err));
    };
    ($fmt:expr, $($arg:tt)*) => {
        return $crate::private::Err($crate::format_err!($fmt, $($arg)*));
    };
}

/// Return early with an error if a condition is not satisfied.
///
/// This macro is equivalent to `if !$cond { return Err(From::from($err)); }`.
///
/// Analogously to `assert!`, `ensure!` takes a condition and exits the function
/// if the condition fails. Unlike `assert!`, `ensure!` returns an `Error`
/// rather than panicking.
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $msg:literal $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::format_err!($msg));
        }
    };
    ($cond:expr, $err:expr $(,)?) => {
        if !$cond {
            return $crate::private::Err($crate::format_err!($err));
        }
    };
    ($cond:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            return $crate::private::Err($crate::format_err!($fmt, $($arg)*));
        }
    };
}

/// Return early with an error if two expressions are not equal to each other.
///
/// This macro is equivalent to `if $left != $right { return Err(From::from($err)); }`.
///
/// Analogously to `assert_eq!`, `ensure_eq!` takes two expressions and exits the function
/// if the expressions are not equal. Unlike `assert_eq!`, `ensure_eq!` returns an `Error`
/// rather than panicking.
#[macro_export]
macro_rules! ensure_eq {
    ($left:expr, $right:expr, $msg:literal $(,)?) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err!($msg));
        }
    };
    ($left:expr, $right:expr, $err:expr $(,)?) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err!($err));
        }
    };
    ($left:expr, $right:expr, $fmt:expr, $($arg:tt)*) => {
        if $left != $right {
            return $crate::private::Err($crate::format_err!($fmt, $($arg)*));
        }
    };
}

/// Construct an ad-hoc error from a string.
///
/// This evaluates to an `Error`. It can take either just a string, or a format
/// string with arguments. It also can take any custom type which implements
/// `Debug` and `Display`.
#[macro_export]
macro_rules! format_err {
    ($msg:literal $(,)?) => {
        // Handle $:literal as a special case to make cargo-expanded code more
        // concise in the common case.
        $crate::private::new_adhoc($msg)
    };
    ($err:expr $(,)?) => ({
        let error = $err;
        Error::new_adhoc(error)
    });
    ($fmt:expr, $($arg:tt)*) => {
        $crate::private::new_adhoc(format!($fmt, $($arg)*))
    };
}
