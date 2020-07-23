#![cfg_attr(feature = "nightly", feature(test))]
#![feature(try_trait)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate bitflags;

#[macro_use]
extern crate derive_more;

#[macro_use]
mod testutils;
#[macro_use]
mod util;
mod ast;
mod ast_types;
mod codecleaner;
mod codeiter;
mod core;
mod fileres;
mod matchers;
#[cfg(feature = "metadata")]
mod metadata;
mod nameres;
mod primitive;
mod project_model;
mod scopes;
mod snippets;
mod typeinf;

pub use crate::ast_types::PathSearch;
pub use crate::core::{
    complete_from_file, complete_fully_qualified_name, find_definition, is_use_stmt, to_coords,
    to_point,
};
pub use crate::core::{
    BytePos, ByteRange, Coordinate, FileCache, FileLoader, Location, Match, MatchType, Session,
};
pub use crate::primitive::PrimKind;
pub use crate::project_model::{Edition, ProjectModelProvider};
pub use crate::snippets::snippet_for_match;
pub use crate::util::expand_ident;

pub use crate::util::{get_rust_src_path, RustSrcPathError};

#[cfg(all(feature = "nightly", test))]
mod benches;
