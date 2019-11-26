Concepts
========

This sections covers the different concepts that Alexandrie defines and uses.  

The goal of these concepts is to formally define how the different parts of Alexandrie interacts.  
Alexandrie can operate with both its crate index and storage on different remote machines and this brings in the need for a well-defined definition of what each of these parts are responsible for.

Alexandrie is built around two principal concepts:

- **Crate index**: manages metadata about published crates.
- **Crate store**: stores actual contents of the published crates (code, assets, etc...).

A **crate index** is a git repository layed out as specified in [**Cargo's Alternative Crate Registries RFC**][Cargo's Alternative Crate Registries RFC].  
It stores metadata about each stored crate and their versions, but not the contents of the crates themselves.  

A **crate store** is what takes care of storing the crate contents themselves and then making them available for download.  

[Cargo's Alternative Crate Registries RFC]: https://github.com/rust-lang/rfcs/blob/master/text/2141-alternative-registries.md
