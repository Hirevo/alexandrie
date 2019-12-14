Available index management strategies
=====================================

'command-line': using the shell 'git' command
---------------------------------------------

This index management strategy invokes the shell 'git' command to manage a local clone of the index's repository.

A limitation of this strategy is that it requires the host machine to have the `git` command installed and available.  

Here is an example and description of a configuration using this index management strategy:

```toml
[index]
type = "command-line" # required.
path = "crate-index"  # required: path of the index's local clone.
```

**NOTE:**  
The local clone must be present and up-to-date before launching Alexandrie.  
Today, Alexandrie won't pull or clone on its own on startup.

'git2': using the `libgit2` library
-----------------------------------

This index management strategy uses `libgit2` to manage a local clone of the index's repository.

The advantage of this strategy over 'command-line' is that it doesn't require `git` to be installed on the host machine.  
The repository interaction is completely independant of the local `git` installation.  

Here is an example and description of a configuration using this index management strategy:

```toml
[index]
type = "git2"         # required.
path = "crate-index"  # required: path of the index's local clone.
```

**NOTE:**  
The local clone must be present and up-to-date before launching Alexandrie.  
Today, Alexandrie won't pull or clone on its own on startup.
