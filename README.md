Alexandrie
==========

This is an alternative crate registry for use with Cargo, written in Rust.

This repository implements the Cargo APIs and interacts with a crate index as specified in the [Cargo's Alternative Registries RFC].  
This allows to have a private registry to host crates that are specific to what your doing and not suitable for publication on [crates.io] while maintaining the same build experience as with crates from [crates.io].  

[crates.io]: https://crates.io
[Cargo's Alternative Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification

Goals
-----

- Offer customizable crate storage strategies (local on-disk, S3, Git Repo, etc...).
- An optional integrated (most-likely server-side rendered) front-end.
- Ideally, customizable backing database options for managing users and crates.
- Do all this while staying performant.

Current state
-------------

- Currently, only MySQL is supported as the backing database.
- Currently, there is no front-end implemented (in-progress).
- Currently, managing users and generating tokens is done manually through the database.

How to build
------------

Alexandrie is built using Rocket and currently requires a MySQL database (MariaDB and PostgreSQL are such databases).  
To build, you can run `cargo build [--release]`.  

To run it, you first have to configure the database credentials in the `Rocket.toml` file:

```toml
[global.databases.alexandrie]
# Replace the '<...>' placeholders by the real ones.
url = "mysql://<user>:<password>@<hostname>:<port>/<database>"
```

Then, you can configure the crate storage strategy and the crate index management strategy that you want to use.  
Here is how to do it (these are also the defaults, you can leave them out if you want):

```toml
[global.crate-index]
type = "cli"
path = "crate-index"

[global.crate-storage]
type = "disk"
path = "crate-storage"
```

You can also configure things like the address and port of the server, the maximum number of workers and other things:

```toml
[development]
address = "localhost"
port = 8000
workers = 1

[staging]
address = "0.0.0.0"
port = 8000
workers = 2

[production]
address = "0.0.0.0"
port = 8000
workers = 4
```

You can refer to [Rocket's configuration documentation] to have the full list of options and what they do.

[Rocket's configuration documentation]: https://rocket.rs/v0.4/guide/configuration/#rockettoml

To run the registry, be sure to clone your crate index at the location designated by the `path` key in `[global.crate-index]`.  
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
index = "<link-to-the-index-git-repository>"
```
- Then, run `cargo login --registry <name-of-your-registry>` and enter your author token.  
  To generate a token, you need to register an author first by creating one in the database.
  You can do it like this:
  ```sql
  insert into `authors` (`login`, `name`, `passwd`) values ("<username>", "<displayable-name>", sha2("<passwd>", 512));
  insert into `author_tokens` (`author_id`, `token`) values (1, sha2(concat(now(), rand(), uuid()), 512));
  select token from `author_tokens` limit 1; -- This will display the token back to you.
  ```
- You can now use the registry using `cargo [search|publish] --registry <name-of-your-registry>`
