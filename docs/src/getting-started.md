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
cargo build [--release] \
    --no-default-features \
    --features [frontend] [sqlite|mysql|postgres]
```

Before running Alexandrie, you'll need to configure your instance in the `alexandrie.toml` file.

The database is configured through the `[database]` table:

```toml
[database]
# Replace the '<...>' placeholders by the real values.

# For MySQL
url = "mysql://<user>:<password>@<hostname>:<port>/<database>"

# For PostgreSQL
url = "postgresql://<user>:<password>@<hostname>:<port>/<database>"

# For SQLite
url = "<path-to-sqlite-file>"
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

To run the registry with the configuration above, be sure to clone your crate index at the location designated by the `path` key in `[index]`.  
In this case, it is `./crate-index`.  
To clone an existing crate index, you can run:

```bash
# Replace the '<...>' placeholders by the real ones.
git clone <url-of-the-crate-index> <path-from-config>
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
  # Replace the '<...>' placeholders by the real ones.
  [registries.<name-of-your-registry>]
  index = "<url-of-the-crate-index>"
  ```
- Then, run `cargo login --registry <name-of-your-registry>` and enter your author token.  
  To generate a token, you need to register as an author first.
  You can do this using the frontend by:
  - Registering at `/account/register`.
  - Generating a token at `/account/manage`.
- You can now use the registry using `cargo [search|publish] --registry <name-of-your-registry>`
