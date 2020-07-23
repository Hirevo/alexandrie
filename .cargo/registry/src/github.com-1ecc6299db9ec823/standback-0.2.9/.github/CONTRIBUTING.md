# Contributing

## Bugs

Think you found a bug? Create an issue on the [issue tracker]. Be sure to include enough information to reproduce the issue, trying to eliminate anything unnecessary.

If an issue already exists for your bug, feel free to add another example! Even if it's similar to an existing one, it might help in narrowing down the cause.

## Pull Requests

All pull requests are appreciated! Even if it's just fixing a typo in documentation, it's an improvement from what we had before. If you're planning on making a larger change to the codebase, it's recommended that you either ask on Matrix or file an issue before doing so - it will save you time if the PR won't be accepted!

### Tests

Any PR modifying code should be accompanied by a change to tests (and possibly additions). Documentation should also be updated if necessary.

### Commits

Commits will almost certainly be squashed when merged, so don't stress about having the perfect commit message. It is appreciated to keep commits to logical chunks, as this makes it easier to review.

## Continuous Integration

standback uses [GitHub Actions] for continuous integration. Types are checked and the full test suite is run on Windows, Mac, and Linux on both the minimum supported Rust version and the most recent stable. Formatting is verified, as are clippy lints.

## Formatting

standback uses rustfmt for formatting, and uses a number of nightly features. As such, you will need to run `cargo +nightly fmt` before committing. Formatting is important, so not doing this may cause CI to fail!

[issue tracker]: https://github.com/jhpratt/standback/issues/new
[GitHub Actions]: https://github.com/features/actions
