Crate index
===========

The **crate index** is a git repository where metadata about all the crates managed by a registry is stored.  
In a way, it is a snapshot of a registry's knowledge of every crates.  

The layout of a crate index is specified in [**Cargo's Alternative Registries RFC**][Cargo's Alternative Registries RFC].  

Alexandrie will create the directories needed to store a crate automatically, so it suffices to only manually create the `config.json` file to get going.  

<!-- TODO: Reformulate RFC's sections about crate index configuration -->

The way the crate index is accessed is called a **crate index management strategy** (a bit of a mouthful, sorry about that 😅).  

Strategies will allow Alexandrie to interact with crate indices in a variety of ways, not only locally but potentially remotely (using a litte server on another machine to perform the operation) where the registry itself doesn't have full access to the underlying git repository.  

[Cargo's Alternative Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification
