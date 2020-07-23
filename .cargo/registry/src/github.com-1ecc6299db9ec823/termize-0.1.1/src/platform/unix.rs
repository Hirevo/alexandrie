// Supress warnings for `TIOCGWINSZ.into()` since freebsd requires it.
#![allow(clippy::identity_conversion)]

use libc::{ioctl, winsize, STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, TIOCGWINSZ};
use std::mem::zeroed;

/// Runs the ioctl command. Returns (0, 0) if all of the streams are not to a terminal, or
/// there is an error. (0, 0) is an invalid size to have anyway, which is why
/// it can be used as a nil value.
unsafe fn get_dimensions_any() -> winsize {
    let mut window: winsize = zeroed();
    let mut result = ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut window);

    if result == -1 {
        window = zeroed();
        result = ioctl(STDIN_FILENO, TIOCGWINSZ.into(), &mut window);
        if result == -1 {
            window = zeroed();
            result = ioctl(STDERR_FILENO, TIOCGWINSZ.into(), &mut window);
            if result == -1 {
                return zeroed();
            }
        }
    }
    window
}

/// Runs the ioctl command. Returns (0, 0) if the output is not to a terminal, or
/// there is an error. (0, 0) is an invalid size to have anyway, which is why
/// it can be used as a nil value.
unsafe fn get_dimensions_out() -> winsize {
    let mut window: winsize = zeroed();
    let result = ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut window);

    if result != -1 {
        return window;
    }
    zeroed()
}

/// Runs the ioctl command. Returns (0, 0) if the input is not to a terminal, or
/// there is an error. (0, 0) is an invalid size to have anyway, which is why
/// it can be used as a nil value.
unsafe fn get_dimensions_in() -> winsize {
    let mut window: winsize = zeroed();
    let result = ioctl(STDIN_FILENO, TIOCGWINSZ.into(), &mut window);

    if result != -1 {
        return window;
    }
    zeroed()
}

/// Runs the ioctl command. Returns (0, 0) if the error is not to a terminal, or
/// there is an error. (0, 0) is an invalid size to have anyway, which is why
/// it can be used as a nil value.
unsafe fn get_dimensions_err() -> winsize {
    let mut window: winsize = zeroed();
    let result = ioctl(STDERR_FILENO, TIOCGWINSZ.into(), &mut window);

    if result != -1 {
        return window;
    }
    zeroed()
}

/// Query the current processes's output (`stdout`), input (`stdin`), and error (`stderr`) in
/// that order, in the attempt to determine terminal width. If one of those streams is actually
/// a tty, this function returns its width and height as a number of characters.
///
/// # Errors
///
/// If *all* of the streams are not ttys or return any errors this function will return `None`.
///
/// # Example
///
/// To get the dimensions of your terminal window, simply use the following:
///
/// ```no_run
/// if let Some((w, h)) = termize::dimensions() {
///     println!("Width: {}\nHeight: {}", w, h);
/// } else {
///     println!("Unable to get term size :(");
/// }
/// ```
pub fn dimensions() -> Option<(usize, usize)> {
    let w = unsafe { get_dimensions_any() };

    if w.ws_col == 0 || w.ws_row == 0 {
        None
    } else {
        Some((w.ws_col as usize, w.ws_row as usize))
    }
}

/// Query the current processes's output (`stdout`) *only*, in the attempt to determine
/// terminal width. If that stream is actually a tty, this function returns its width
/// and height as a number of characters.
///
/// # Errors
///
/// If the stream is not a tty or return any errors this function will return `None`.
///
/// # Example
///
/// To get the dimensions of your terminal window, simply use the following:
///
/// ```no_run
/// if let Some((w, h)) = termize::dimensions_stdout() {
///     println!("Width: {}\nHeight: {}", w, h);
/// } else {
///     println!("Unable to get term size :(");
/// }
/// ```
pub fn dimensions_stdout() -> Option<(usize, usize)> {
    let w = unsafe { get_dimensions_out() };

    if w.ws_col == 0 || w.ws_row == 0 {
        None
    } else {
        Some((w.ws_col as usize, w.ws_row as usize))
    }
}

/// Query the current processes's input (`stdin`) *only*, in the attempt to determine
/// terminal width. If that stream is actually a tty, this function returns its width
/// and height as a number of characters.
///
/// # Errors
///
/// If the stream is not a tty or return any errors this function will return `None`.
///
/// # Example
///
/// To get the dimensions of your terminal window, simply use the following:
///
/// ```no_run
/// if let Some((w, h)) = termize::dimensions_stdin() {
///     println!("Width: {}\nHeight: {}", w, h);
/// } else {
///     println!("Unable to get term size :(");
/// }
/// ```
pub fn dimensions_stdin() -> Option<(usize, usize)> {
    let w = unsafe { get_dimensions_in() };

    if w.ws_col == 0 || w.ws_row == 0 {
        None
    } else {
        Some((w.ws_col as usize, w.ws_row as usize))
    }
}

/// Query the current processes's error output (`stderr`) *only*, in the attempt to dtermine
/// terminal width. If that stream is actually a tty, this function returns its width
/// and height as a number of characters.
///
/// # Errors
///
/// If the stream is not a tty or return any errors this function will return `None`.
///
/// # Example
///
/// To get the dimensions of your terminal window, simply use the following:
///
/// ```no_run
/// if let Some((w, h)) = termize::dimensions_stderr() {
///     println!("Width: {}\nHeight: {}", w, h);
/// } else {
///     println!("Unable to get term size :(");
/// }
/// ```
pub fn dimensions_stderr() -> Option<(usize, usize)> {
    let w = unsafe { get_dimensions_err() };

    if w.ws_col == 0 || w.ws_row == 0 {
        None
    } else {
        Some((w.ws_col as usize, w.ws_row as usize))
    }
}

#[cfg(test)]
mod tests {
    use super::dimensions;
    use std::process::{Command, Output, Stdio};

    #[cfg(target_os = "macos")]
    fn stty_size() -> Output {
        Command::new("stty")
            .arg("-f")
            .arg("/dev/stderr")
            .arg("size")
            .stderr(Stdio::inherit())
            .output()
            .unwrap()
    }

    #[cfg(not(target_os = "macos"))]
    fn stty_size() -> Output {
        Command::new("stty")
            .arg("-F")
            .arg("/dev/stderr")
            .arg("size")
            .stderr(Stdio::inherit())
            .output()
            .expect("failed to run `stty_size()`")
    }

    #[test]
    fn test_shell() {
        let output = stty_size();
        let stdout = String::from_utf8(output.stdout).expect("failed to turn into String");
        let mut data = stdout.split_whitespace();
        let rs = data
            .next()
            .unwrap_or("0")
            .parse::<usize>()
            .expect("failed to parse rows");
        let cs = data
            .next()
            .unwrap_or("0")
            .parse::<usize>()
            .expect("failed to parse cols");
        println!("stdout: {}", stdout);
        println!("rows: {}\ncols: {}", rs, cs);
        if let Some((w, h)) = dimensions() {
            assert_eq!(rs, h);
            assert_eq!(cs, w);
        }
    }
}
