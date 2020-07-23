# Version 0.1.11

- Update `wepoll-binding`.
- Reduce dependencies.
- Replace `nix` with `libc`.
- Set minimum required `tokio` version to 0.2.

# Version 0.1.10

- Fix incorrectly reported error kind when connecting fails.

# Version 0.1.9

- Switch to oneshot-style notifications on all platforms.
- Fix a bug that caused 100% CPU usage on Windows.
- Deprecate `Async::with()` and `Async::with_mut()`.
- Add `Async::read_with()`, `Async::read_with_mut()`,
  `Async::write_with()`, and `Async::write_with_mut()`.
- Fix a bug where eventfd was not closed.

# Version 0.1.8

- Revert the use of `blocking` crate.

# Version 0.1.7

- Update `blocking` to `0.4.2`.
- Make `Task::blocking()` work without `run()`.

# Version 0.1.6

- Fix a deadlock by always re-registering `IoEvent`.

# Version 0.1.5

- Use `blocking` crate for blocking I/O.
- Fix a re-registration bug when in oneshot mode.
- Use eventfd on Linux.
- More tests.
- Fix timeout rounding error in epoll/wepoll.

# Version 0.1.4

- Fix a bug in UDS async connect

# Version 0.1.3

- Fix the writability check in async connect
- More comments and documentation
- Better security advice on certificates

# Version 0.1.2

- Improved internal docs, fixed typos, and more comments

# Version 0.1.1

- Upgrade dependencies

# Version 0.1.0

- Initial release
