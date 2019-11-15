Crate index
===========

The crate index is a git repository where metadata about all the crates managed by a registry is stored.  
In a way, it is a snapshot of a registry's knowledge of every crates.  

The layout of a crate index is specified in [**Cargo's Alternative Registries RFC**][Cargo's Alternative Registries RFC].  

Alexandrie will create the directories needed to store a crate automatically, so it suffices to only manually create the `config.json` file to get going.  

<!-- TODO: Reformulate RFC's sections about crate index configuration -->

[Cargo's Alternative Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md#registry-index-format-specification
