#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::{dimensions, dimensions_stderr, dimensions_stdin, dimensions_stdout};

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::{dimensions, dimensions_stderr, dimensions_stdin, dimensions_stdout};

// makes project compilable on unsupported platforms
#[cfg(not(any(unix, windows)))]
mod unsupported;
#[cfg(not(any(unix, windows)))]
pub use self::unsupported::{dimensions, dimensions_stderr, dimensions_stdin, dimensions_stdout};
