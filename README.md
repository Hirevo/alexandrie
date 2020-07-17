<div align=center><h1>Alexandrie</h1></div>
<div align=center><strong>Modular alternative crate registry for Rust</strong></div>

About
-----

Alexandrie is an alternative crate registry suitable for use with Cargo.

This repository implements the Cargo APIs and interacts with a crate index as specified in the [Cargo's Alternative Registries RFC].  
This allows to have a private registry to host crates that are specific to what you're doing and not suitable for publication on [crates.io] while maintaining the same build experience as with crates from [crates.io].  

[crates.io]: https://crates.io
[Cargo's Alternative Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification

Goals
-----

- Offer customizable crate storage strategies (local on-disk, S3, Git Repo, etc...).
- Offer multiple backing database options (MySQL, PostgreSQL or SQLite).
- An optional integrated (server-side rendered) front-end.

Current state
-------------

- The core Cargo APIs are all functional.
- The optional front-end is very usable, although still in active development.

Things yet to do
----------------

- [ ] Complete the front-end: in-progress
- [x] Keywords: done
- [x] Categories: done
- [x] Crate (un)yanking: done
- [x] User management: done
- [ ] Crate version tracking in DB (download counts per version, etc...): planned
- [ ] Ability to re-render READMEs (to migrate themes): planned
- [ ] Search by keywords or categories: planned
- [ ] More `Store` implementors: planned
- [ ] More `Indexer` implementors: planned

How to build
------------

Alexandrie is built using [**Tide**][Tide] and offers multiple options to be used as its database.  
To build, you can run `cargo build [--release]`.  

[Tide]: https://github.com/http-rs/tide

Before running it, you need to configure your instance in the `alexandrie.toml` file.

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

Optionally, you can specify the maximum number of simultaneous open connections for the database connection pool:

```toml
max_conns = 1
```

If not specified, the `r2d2` [default value](https://docs.diesel.rs/diesel/r2d2/struct.Builder.html#method.max_size) is used.

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

Then, you need to configure a crate index.  
A crate index is simply a git repository that the registry uses to keep metadata information about each crate and their individual versions.  
The repository can be created on any machine you want, as long as it is reachable from the current machine as a git remote in a clone of that repository.  
The remote can be specified using either an HTTPS or SSH link.  

_If you're using SSH for the remote link, Cargo might have an issue where it can't fetch from the registry's index when doing `cargo search` or `cargo build`._  
_This is because Cargo uses `libgit2` to fetch from remotes and fails to find the SSH credentials needed to do so._  
_To work around this issue, you may need to set the `CARGO_NET_GIT_FETCH_WITH_CLI` environment variable to `true`, so that Cargo will offload the fetching from remotes operation to the `git` command-line utility._  
_See [issue #44](https://github.com/Hirevo/alexandrie/issues/44) for a previous occurence of this exact issue._  

To run the registry, be sure to clone your crate index at the location designated by the `path` key in `[index]`.  
The default for it is `./crate-index`.  
To clone an existing crate index, you can run:

```bash
# Replace the '<...>' placeholders by their real actual values.
git clone <url-of-the-crate-index> <path-from-config>

# <url-of-the-crate-index>: URL to the git repository serving as the registry's crate index.
# <path-from-config>: Path to the same directory as the one specified as `index.path` in the `alexandrie.toml`.

# Example:
git clone 'https://github.com/Hirevo/alexandrie-index' 'crate-index'
```

If you want to create one, you can refer to the [Cargo's Alternative Registries RFC] to learn about the layout of such an index.  
You can also visit the [crates.io index] or the [crates.polomack.eu index] as deployed examples.  

[crates.io index]: https://github.com/rust-lang/crates.io-index
[crates.polomack.eu index]: https://github.com/Hirevo/alexandrie-index

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

Installation script
-------------------

If you wish to have a more concrete resource to learn how to setup an Alexandrie instance, like a shell script, you may refer to an example installation script which can help you get started:

**<https://hirevo.github.io/alexandrie/installation-script.html>**

Docker Compose
--------------

You can host Alexandrie in a Docker container on your computer or a host machine of your choosing. You will need both [docker](https://docs.docker.com/install/) and [docker-compose](https://docs.docker.com/compose/install/) installed.

To get started, you'll need to copy the `example.env` file and save it as `.env` (filename is important). You should:

- Set `APPDATA` to the path of a new directory where the container will store the crate index, crate files, & database file.
- Set `CRATE_INDEX` to the SSH path of an existing repo with a valid index `config.json` file.
- Set `GIT_NAME` and `GIT_EMAIL` to valid git values that will be used when Alexandrie commits & pushes those commits to the index.
- Set `GIT_SSH_KEY` to a new or existing passwordless SSH key. The `.pub` key associated with this key should be added to GitHub/GitLab/etc. to grant access to push to the crate index.

These items will be mounted into the Docker container, and need to be accessible by a user with UID and GID `1000`. If Docker appears to complain that any of these are inaccessible, check your paths and your file/directory permissions.

By default, Alexandrie will use SQLite for its database. If you want to use either MySQL or PostgreSQL instead, you'll need to create a `rootpass.txt` in `docker/<database>/`. The entire contents of this file will be copied and used as the password for the root user of the database; don't add an ending newline unless your password actually contains one!

To run Alexandrie, call the `run_docker.sh` script, with arguments depending on the action and database you want. For example, to start Alexandrie in the background with the default SQLite database, do:

```bash
./run_docker.sh up
```

To stop Alexandrie, do:

```bash
./run_docker.sh down
```

The script assumes a Bash environment, and was only tested on Ubuntu 19.10. For more details and examples, see the docs.

License
-------

Licensed under either of

- Apache License, Version 2.0 (LICENSE-APACHE or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license (LICENSE-MIT or <http://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
