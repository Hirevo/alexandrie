Crate storage
=============

A **crate store** is the place where all the contents of the published crates (the actual code and assets) are stored.  

The contents of crates are stored as TAR archives and compressed using Gzip (basically a `.tar.gz` blob).  
The store takes that blob, stores it and make it available for download later on.  

Currently, the store is also responsible for storing rendered README pages (which are simple HTML files).  

Because these can amount to a lot of storage space, it can be desirable to separate the crates' metadata (modelled by the crate index) and their actual contents (handled by the crate stores).  

A crate store may be local (as files on disk, for instance) or remote (as blobs in AWS S3, for instance).
For usage and configuration see [Crate stores](../whats-available/crate-stores.md).
