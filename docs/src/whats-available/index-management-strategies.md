Available index management strategies
=====================================

'cli': using the shell 'git' command
------------------------------------

This index management strategy invokes the shell 'git' command to manage a local clone of the index's repository.

Here is an example and description of a configuration using this index management strategy:

```toml
[index]
type = "cli"          # required.
path = "crate-index"  # required: path of the index's local clone.
```

**NOTE:**  
The local clone must be present and up-to-date before launching Alexandrie.  
Today, Alexandrie won't pull or clone on its own on startup.
