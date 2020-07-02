/// Index management through `git` shell command invocations.
pub mod cli;

/// Index management using [**`libgit2`**][libgit2].
/// [libgit2]: https://libgit2.org
#[cfg(feature = "git2")]
pub mod git2;
