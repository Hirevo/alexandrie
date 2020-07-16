Getting started
===============

How to build
------------

Alexandrie offers multiple options to be used as its database, so before building Alexandrie, you'll need to pick which supported database vendor you wish to use.  
The currently supported vendors are:

- **`sqlite`**: for SQLite
- **`mysql`**: for MySQL (including MariaDB)
- **`postgres`**: for PostgreSQL

To build, you can then run:

```bash
# navigate to the `alexandrie/` subfolder
cd alexandrie

# build the binary
# (replace `(foo|bar)` occurences by one of `foo` or `bar`)
# (replace `[foo]` occurences by either `foo` or nothing)
cargo build [--release] \
    --no-default-features \
    --features "[frontend] (sqlite|mysql|postgres)"
```

Before running Alexandrie, you'll need to configure your instance in the `alexandrie.toml` file.

The database is configured through the `[database]` table:

```toml
[database]
# Replace the '<...>' placeholders by their real actual values.

# For MySQL
url = "mysql://<user>:<password>@<hostname>:<port>/<database>"

# For PostgreSQL
url = "postgresql://<user>:<password>@<hostname>:<port>/<database>"

# For SQLite
url = "<path-to-sqlite-database-file>"
# or:
url = ":memory:" # ephemeral in-memory database, doesn't persists between restarts
```

Then, you can configure the crates' tarballs storage strategy and the crate index management strategy that you want to use.  

Here is an example of how to do it:

```toml
[index]
type = "command-line"
path = "crate-index"

[storage]
type = "disk"
path = "crate-storage"
```

You can find more information about crate index management and crate stores in the **'What's available'** section of this book.

You can also configure things like the address and port of the server:

```toml
[general]
addr = "127.0.0.1"  # Endpoint on which to serve the service.
port = 3000         # Port on which to serve the service.
```

Then, you need to configure a crate index.  
A crate index is simply a git repository that the registry uses to keep metadata information about each crate and their individual versions.  
The repository can be created on any machine you want, as long as it is reachable from the current machine as a git remote in a clone of that repository.  
The remote can be specified using either an HTTPS or SSH link.  

_If you're using SSH for the remote link, Cargo might have an issue where it can't fetch from the registry's index when doing `cargo search` or `cargo build`._  
_This is because Cargo uses `libgit2` to fetch from remotes and fails to find the SSH credentials needed to do so._  
_To work around this issue, you may need to set the `CARGO_NET_GIT_FETCH_WITH_CLI` environment variable to `true`, so that Cargo will offload the fetching from remotes operation to the `git` command-line utility._  
_See [issue #44](https://github.com/Hirevo/alexandrie/issues/44) for a previous occurence of this exact issue._  

To run the registry with the configuration above, be sure to clone your crate index at the location designated by the `path` key in `[index]`.  
In this case, it is `./crate-index`.  
To clone an existing crate index, you can run:

```bash
# Replace the '<...>' placeholders by their real actual values.
git clone <url-of-the-crate-index> <path-from-config>

# <url-of-the-crate-index>: URL to the git repository serving as the registry's crate index.
# <path-from-config>: Path to the same directory as the one specified as `index.path` in the `alexandrie.toml`.

# Example:
git clone 'https://github.com/Hirevo/alexandrie-index' 'crate-index'
```

If you want to create one, you can refer to the [**Cargo's Alternative Registries RFC**][Cargo's Alternative Registries RFC] to learn about the layout of such an index.  
You can also visit the [**crates.io index**][crates.io index] or the [**crates.polomack.eu index**][crates.polomack.eu index] as deployed examples.  

[Cargo's Alternative Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification
[crates.io index]: https://github.com/rust-lang/crates.io-index
[crates.polomack.eu index]: https://github.com/Hirevo/alexandrie-index

How to run
----------

Once everything is configured, you can run with: `cargo run [--release]`.

Then, if you want to use this index with Cargo, you can follow these steps:

- Edit or create the `~/.cargo/config` file, and add the following code:
  ```toml
  # Replace the '<...>' placeholders by their real actual values.
  [registries.<name-of-your-registry>]
  index = "<url-of-the-crate-index>"

  # <name-of-your-registry>: A name of your choosing, that you'll be using to refer to it in `cargo` commands.
  # <url-of-the-crate-index>: URL to the git repository serving as the registry's crate index.
  #                           BE CAREFUL: this is not the URL to the registry's API or frontend.
  ```
- Then, run `cargo login --registry <name-of-your-registry>` and enter your author token.  
  To generate a token, you need to register as an author first.
  You can do this using the frontend by:
  - Registering at `/account/register`.
  - Generating a token at `/account/manage`.
- You can now use the registry using `cargo [search|publish] --registry <name-of-your-registry>`
