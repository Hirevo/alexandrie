//! The Rust parser and macro expander.
//!
//! # Note
//!
//! This API is completely unstable and subject to change.

#![doc(html_root_url = "https://doc.rust-lang.org/nightly/", test(attr(deny(warnings))))]
#![feature(rustc_private, bool_to_option)]
#![feature(box_syntax)]
#![feature(const_if_match)]
#![feature(const_fn)] // For the `transmute` in `P::new`
#![feature(const_panic)]
#![feature(const_transmute)]
#![feature(crate_visibility_modifier)]
#![feature(label_break_value)]
#![feature(nll)]
#![feature(or_patterns)]
#![feature(try_trait)]
#![feature(unicode_internals)]
#![recursion_limit = "256"]

// FIXME(#56935): Work around ICEs during cross-compilation.
#[allow(unused)]
extern crate rustc_macros;

#[macro_export]
macro_rules! unwrap_or {
    ($opt:expr, $default:expr) => {
        match $opt {
            Some(x) => x,
            None => $default,
        }
    };
}

pub mod util {
    pub mod classify;
    pub mod comments;
    pub mod lev_distance;
    pub mod literal;
    pub mod parser;
}

pub mod ast;
pub mod attr;
pub use attr::{with_default_globals, with_globals, GLOBALS};
pub mod crate_disambiguator;
pub mod entry;
pub mod expand;
pub mod mut_visit;
pub mod node_id;
pub mod ptr;
pub mod token;
pub mod tokenstream;
pub mod visit;

use rustc_data_structures::stable_hasher::{HashStable, StableHasher};

/// Requirements for a `StableHashingContext` to be used in this crate.
/// This is a hack to allow using the `HashStable_Generic` derive macro
/// instead of implementing everything in librustc_middle.
pub trait HashStableContext: rustc_span::HashStableContext {
    fn hash_attr(&mut self, _: &ast::Attribute, hasher: &mut StableHasher);
}

impl<AstCtx: crate::HashStableContext> HashStable<AstCtx> for ast::Attribute {
    fn hash_stable(&self, hcx: &mut AstCtx, hasher: &mut StableHasher) {
        hcx.hash_attr(self, hasher)
    }
}
