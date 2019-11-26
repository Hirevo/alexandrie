Available crate stores
======================

'disk': Local on-disk store
---------------------------

This store implements simple local storage of crates as files in a given directory.

Here is an example of a configuration using this storage:

```toml
[storage]
type = "disk"           # required.
path = "crate-storage"  # required: path of the directory in which to store the crates.
```
