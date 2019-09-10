<div align=center><h1>Alexandrie</h1></div>
<div align=center><strong>Modular alternative crate registry for Rust</strong></div>

About
-----

This is an alternative crate registry for use with Cargo, written in Rust.

This repository implements the Cargo APIs and interacts with a crate index as specified in the [Cargo's Alternative Registries RFC].  
This allows to have a private registry to host crates that are specific to what your doing and not suitable for publication on [crates.io] while maintaining the same build experience as with crates from [crates.io].  

[crates.io]: https://crates.io
[Cargo's Alternative Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification

Goals
-----

- Offer customizable crate storage strategies (local on-disk, S3, Git Repo, etc...).
- An optional integrated (server-side rendered) front-end.
- Be performant.

Current state
-------------

- The core Cargo APIs are functional but not yet complete.
- The optional front-end is in active development.
- Currently, generating tokens is done manually through the database.

How to build
------------

Alexandrie is built using Tide and currently requires a MySQL database (MariaDB and PostgreSQL are such databases).  
To build, you can run `cargo build [--release]`.  

Before running it, you need to configure the instance in the `alexandrie.toml` file.

The database is configured through the `[database]` table:

```toml
[database]
# Replace the '<...>' placeholders by the real ones.
url = "mysql://<user>:<password>@<hostname>:<port>/<database>"
```

Then, you can configure the crates' tarballs storage strategy and the crate index management strategy that you want to use.  
Here is how to do it (these are also the defaults, you can leave them out if you want):

```toml
[index]
type = "command-line"
path = "crate-index"

[storage]
type = "disk"
path = "crate-storage"
```

You can also configure things like the address and port of the server:

```toml
[general]
addr = "127.0.0.1"
port = 3000
```

To run the registry, be sure to clone your crate index at the location designated by the `path` key in `[index]`.  
The default for it is `./crate-index`.  
To clone an existing crate index, you can run:

```bash
# Replace the '<...>' placeholders by the real ones.
git clone <url-of-the-crate-index> <path-from-config>
```

If you want to create one, you can refer to the [Cargo's Alternative Registries RFC] to learn about the layout of such an index.  
You can also visit the [crates.io index] or the [crates.polomack.eu index] as deployed examples.  

[crates.io index]: https://github.com/rust-lang/crates.io-index
[crates.polomack.eu index]: https://github.com/Hirevo/alexandrie-index

Once everything is configured, you can run with: `cargo run [--release]`.

Then, if you want to use this index with Cargo, you can follow these steps:

- Edit or create the `~/.cargo/config` file, and add the following code:
  ```toml
  # Replace the '<...>' placeholders by the real ones.
  [registries.<name-of-your-registry>]
  index = "<url-of-the-crate-index>"
  ```
- Then, run `cargo login --registry <name-of-your-registry>` and enter your author token.  
  To generate a token, you need to register an author first by creating one in the database.
  You can do it like this:
  ```sql
  -- Replace the '<...>' placeholders by the real ones.
  insert into `authors` (`email`, `name`, `passwd`) values ("<email>", "<displayable-name>", sha2("<passwd>", 512));
  insert into `author_tokens` (`author_id`, `token`) values (1, sha2(concat(now(), rand(), uuid()), 512));
  select token from `author_tokens` limit 1; -- This will display the token back to you.
  ```
- You can now use the registry using `cargo [search|publish] --registry <name-of-your-registry>`
