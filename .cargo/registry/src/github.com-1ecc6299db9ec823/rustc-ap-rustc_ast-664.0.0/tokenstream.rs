//! # Token Streams
//!
//! `TokenStream`s represent syntactic objects before they are converted into ASTs.
//! A `TokenStream` is, roughly speaking, a sequence (eg stream) of `TokenTree`s,
//! which are themselves a single `Token` or a `Delimited` subsequence of tokens.
//!
//! ## Ownership
//!
//! `TokenStream`s are persistent data structures constructed as ropes with reference
//! counted-children. In general, this means that calling an operation on a `TokenStream`
//! (such as `slice`) produces an entirely new `TokenStream` from the borrowed reference to
//! the original. This essentially coerces `TokenStream`s into 'views' of their subparts,
//! and a borrowed `TokenStream` is sufficient to build an owned `TokenStream` without taking
//! ownership of the original.

use crate::token::{self, DelimToken, Token, TokenKind};

use rustc_data_structures::stable_hasher::{HashStable, StableHasher};
use rustc_data_structures::sync::Lrc;
use rustc_macros::HashStable_Generic;
use rustc_span::{Span, DUMMY_SP};
use smallvec::{smallvec, SmallVec};

use log::debug;

use std::{iter, mem};

/// When the main rust parser encounters a syntax-extension invocation, it
/// parses the arguments to the invocation as a token-tree. This is a very
/// loose structure, such that all sorts of different AST-fragments can
/// be passed to syntax extensions using a uniform type.
///
/// If the syntax extension is an MBE macro, it will attempt to match its
/// LHS token tree against the provided token tree, and if it finds a
/// match, will transcribe the RHS token tree, splicing in any captured
/// `macro_parser::matched_nonterminals` into the `SubstNt`s it finds.
///
/// The RHS of an MBE macro is the only place `SubstNt`s are substituted.
/// Nothing special happens to misnamed or misplaced `SubstNt`s.
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable, HashStable_Generic)]
pub enum TokenTree {
    /// A single token
    Token(Token),
    /// A delimited sequence of token trees
    Delimited(DelimSpan, DelimToken, TokenStream),
}

// Ensure all fields of `TokenTree` is `Send` and `Sync`.
#[cfg(parallel_compiler)]
fn _dummy()
where
    Token: Send + Sync,
    DelimSpan: Send + Sync,
    DelimToken: Send + Sync,
    TokenStream: Send + Sync,
{
}

impl TokenTree {
    /// Checks if this TokenTree is equal to the other, regardless of span information.
    pub fn eq_unspanned(&self, other: &TokenTree) -> bool {
        match (self, other) {
            (TokenTree::Token(token), TokenTree::Token(token2)) => token.kind == token2.kind,
            (TokenTree::Delimited(_, delim, tts), TokenTree::Delimited(_, delim2, tts2)) => {
                delim == delim2 && tts.eq_unspanned(&tts2)
            }
            _ => false,
        }
    }

    // See comments in `Nonterminal::to_tokenstream` for why we care about
    // *probably* equal here rather than actual equality
    //
    // This is otherwise the same as `eq_unspanned`, only recursing with a
    // different method.
    pub fn probably_equal_for_proc_macro(&self, other: &TokenTree) -> bool {
        match (self, other) {
            (TokenTree::Token(token), TokenTree::Token(token2)) => {
                token.probably_equal_for_proc_macro(token2)
            }
            (TokenTree::Delimited(_, delim, tts), TokenTree::Delimited(_, delim2, tts2)) => {
                delim == delim2 && tts.probably_equal_for_proc_macro(&tts2)
            }
            _ => false,
        }
    }

    /// Retrieves the TokenTree's span.
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Token(token) => token.span,
            TokenTree::Delimited(sp, ..) => sp.entire(),
        }
    }

    /// Modify the `TokenTree`'s span in-place.
    pub fn set_span(&mut self, span: Span) {
        match self {
            TokenTree::Token(token) => token.span = span,
            TokenTree::Delimited(dspan, ..) => *dspan = DelimSpan::from_single(span),
        }
    }

    pub fn joint(self) -> TokenStream {
        TokenStream::new(vec![(self, Joint)])
    }

    pub fn token(kind: TokenKind, span: Span) -> TokenTree {
        TokenTree::Token(Token::new(kind, span))
    }

    /// Returns the opening delimiter as a token tree.
    pub fn open_tt(span: DelimSpan, delim: DelimToken) -> TokenTree {
        TokenTree::token(token::OpenDelim(delim), span.open)
    }

    /// Returns the closing delimiter as a token tree.
    pub fn close_tt(span: DelimSpan, delim: DelimToken) -> TokenTree {
        TokenTree::token(token::CloseDelim(delim), span.close)
    }

    pub fn uninterpolate(self) -> TokenTree {
        match self {
            TokenTree::Token(token) => TokenTree::Token(token.uninterpolate().into_owned()),
            tt => tt,
        }
    }
}

impl<CTX> HashStable<CTX> for TokenStream
where
    CTX: crate::HashStableContext,
{
    fn hash_stable(&self, hcx: &mut CTX, hasher: &mut StableHasher) {
        for sub_tt in self.trees() {
            sub_tt.hash_stable(hcx, hasher);
        }
    }
}

/// A `TokenStream` is an abstract sequence of tokens, organized into `TokenTree`s.
///
/// The goal is for procedural macros to work with `TokenStream`s and `TokenTree`s
/// instead of a representation of the abstract syntax tree.
/// Today's `TokenTree`s can still contain AST via `token::Interpolated` for back-compat.
#[derive(Clone, Debug, Default, RustcEncodable, RustcDecodable)]
pub struct TokenStream(pub Lrc<Vec<TreeAndJoint>>);

pub type TreeAndJoint = (TokenTree, IsJoint);

// `TokenStream` is used a lot. Make sure it doesn't unintentionally get bigger.
#[cfg(target_arch = "x86_64")]
rustc_data_structures::static_assert_size!(TokenStream, 8);

#[derive(Clone, Copy, Debug, PartialEq, RustcEncodable, RustcDecodable)]
pub enum IsJoint {
    Joint,
    NonJoint,
}

use IsJoint::*;

impl TokenStream {
    /// Given a `TokenStream` with a `Stream` of only two arguments, return a new `TokenStream`
    /// separating the two arguments with a comma for diagnostic suggestions.
    pub fn add_comma(&self) -> Option<(TokenStream, Span)> {
        // Used to suggest if a user writes `foo!(a b);`
        let mut suggestion = None;
        let mut iter = self.0.iter().enumerate().peekable();
        while let Some((pos, ts)) = iter.next() {
            if let Some((_, next)) = iter.peek() {
                let sp = match (&ts, &next) {
                    (_, (TokenTree::Token(Token { kind: token::Comma, .. }), _)) => continue,
                    (
                        (TokenTree::Token(token_left), NonJoint),
                        (TokenTree::Token(token_right), _),
                    ) if ((token_left.is_ident() && !token_left.is_reserved_ident())
                        || token_left.is_lit())
                        && ((token_right.is_ident() && !token_right.is_reserved_ident())
                            || token_right.is_lit()) =>
                    {
                        token_left.span
                    }
                    ((TokenTree::Delimited(sp, ..), NonJoint), _) => sp.entire(),
                    _ => continue,
                };
                let sp = sp.shrink_to_hi();
                let comma = (TokenTree::token(token::Comma, sp), NonJoint);
                suggestion = Some((pos, comma, sp));
            }
        }
        if let Some((pos, comma, sp)) = suggestion {
            let mut new_stream = vec![];
            let parts = self.0.split_at(pos + 1);
            new_stream.extend_from_slice(parts.0);
            new_stream.push(comma);
            new_stream.extend_from_slice(parts.1);
            return Some((TokenStream::new(new_stream), sp));
        }
        None
    }
}

impl From<TokenTree> for TokenStream {
    fn from(tree: TokenTree) -> TokenStream {
        TokenStream::new(vec![(tree, NonJoint)])
    }
}

impl From<TokenTree> for TreeAndJoint {
    fn from(tree: TokenTree) -> TreeAndJoint {
        (tree, NonJoint)
    }
}

impl iter::FromIterator<TokenTree> for TokenStream {
    fn from_iter<I: IntoIterator<Item = TokenTree>>(iter: I) -> Self {
        TokenStream::new(iter.into_iter().map(Into::into).collect::<Vec<TreeAndJoint>>())
    }
}

impl Eq for TokenStream {}

impl PartialEq<TokenStream> for TokenStream {
    fn eq(&self, other: &TokenStream) -> bool {
        self.trees().eq(other.trees())
    }
}

impl TokenStream {
    pub fn new(streams: Vec<TreeAndJoint>) -> TokenStream {
        TokenStream(Lrc::new(streams))
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn span(&self) -> Option<Span> {
        match &**self.0 {
            [] => None,
            [(tt, _)] => Some(tt.span()),
            [(tt_start, _), .., (tt_end, _)] => Some(tt_start.span().to(tt_end.span())),
        }
    }

    pub fn from_streams(mut streams: SmallVec<[TokenStream; 2]>) -> TokenStream {
        match streams.len() {
            0 => TokenStream::default(),
            1 => streams.pop().unwrap(),
            _ => {
                // We are going to extend the first stream in `streams` with
                // the elements from the subsequent streams. This requires
                // using `make_mut()` on the first stream, and in practice this
                // doesn't cause cloning 99.9% of the time.
                //
                // One very common use case is when `streams` has two elements,
                // where the first stream has any number of elements within
                // (often 1, but sometimes many more) and the second stream has
                // a single element within.

                // Determine how much the first stream will be extended.
                // Needed to avoid quadratic blow up from on-the-fly
                // reallocations (#57735).
                let num_appends = streams.iter().skip(1).map(|ts| ts.len()).sum();

                // Get the first stream. If it's `None`, create an empty
                // stream.
                let mut iter = streams.drain(..);
                let mut first_stream_lrc = iter.next().unwrap().0;

                // Append the elements to the first stream, after reserving
                // space for them.
                let first_vec_mut = Lrc::make_mut(&mut first_stream_lrc);
                first_vec_mut.reserve(num_appends);
                for stream in iter {
                    first_vec_mut.extend(stream.0.iter().cloned());
                }

                // Create the final `TokenStream`.
                TokenStream(first_stream_lrc)
            }
        }
    }

    pub fn trees(&self) -> Cursor {
        self.clone().into_trees()
    }

    pub fn into_trees(self) -> Cursor {
        Cursor::new(self)
    }

    /// Compares two `TokenStream`s, checking equality without regarding span information.
    pub fn eq_unspanned(&self, other: &TokenStream) -> bool {
        let mut t1 = self.trees();
        let mut t2 = other.trees();
        for (t1, t2) in t1.by_ref().zip(t2.by_ref()) {
            if !t1.eq_unspanned(&t2) {
                return false;
            }
        }
        t1.next().is_none() && t2.next().is_none()
    }

    // See comments in `Nonterminal::to_tokenstream` for why we care about
    // *probably* equal here rather than actual equality
    //
    // This is otherwise the same as `eq_unspanned`, only recursing with a
    // different method.
    pub fn probably_equal_for_proc_macro(&self, other: &TokenStream) -> bool {
        // When checking for `probably_eq`, we ignore certain tokens that aren't
        // preserved in the AST. Because they are not preserved, the pretty
        // printer arbitrarily adds or removes them when printing as token
        // streams, making a comparison between a token stream generated from an
        // AST and a token stream which was parsed into an AST more reliable.
        fn semantic_tree(tree: &TokenTree) -> bool {
            if let TokenTree::Token(token) = tree {
                if let
                    // The pretty printer tends to add trailing commas to
                    // everything, and in particular, after struct fields.
                    | token::Comma
                    // The pretty printer emits `NoDelim` as whitespace.
                    | token::OpenDelim(DelimToken::NoDelim)
                    | token::CloseDelim(DelimToken::NoDelim)
                    // The pretty printer collapses many semicolons into one.
                    | token::Semi
                    // The pretty printer collapses whitespace arbitrarily and can
                    // introduce whitespace from `NoDelim`.
                    | token::Whitespace
                    // The pretty printer can turn `$crate` into `::crate_name`
                    | token::ModSep = token.kind {
                    return false;
                }
            }
            true
        }

        // When comparing two `TokenStream`s, we ignore the `IsJoint` information.
        //
        // However, `rustc_parse::lexer::tokentrees::TokenStreamBuilder` will
        // use `Token.glue` on adjacent tokens with the proper `IsJoint`.
        // Since we are ignoreing `IsJoint`, a 'glued' token (e.g. `BinOp(Shr)`)
        // and its 'split'/'unglued' compoenents (e.g. `Gt, Gt`) are equivalent
        // when determining if two `TokenStream`s are 'probably equal'.
        //
        // Therefore, we use `break_two_token_op` to convert all tokens
        // to the 'unglued' form (if it exists). This ensures that two
        // `TokenStream`s which differ only in how their tokens are glued
        // will be considered 'probably equal', which allows us to keep spans.
        //
        // This is important when the original `TokenStream` contained
        // extra spaces (e.g. `f :: < Vec < _ > > ( ) ;'). These extra spaces
        // will be omitted when we pretty-print, which can cause the original
        // and reparsed `TokenStream`s to differ in the assignment of `IsJoint`,
        // leading to some tokens being 'glued' together in one stream but not
        // the other. See #68489 for more details.
        fn break_tokens(tree: TokenTree) -> impl Iterator<Item = TokenTree> {
            // In almost all cases, we should have either zero or one levels
            // of 'unglueing'. However, in some unusual cases, we may need
            // to iterate breaking tokens mutliple times. For example:
            // '[BinOpEq(Shr)] => [Gt, Ge] -> [Gt, Gt, Eq]'
            let mut token_trees: SmallVec<[_; 2]>;
            if let TokenTree::Token(token) = &tree {
                let mut out = SmallVec::<[_; 2]>::new();
                out.push(token.clone());
                // Iterate to fixpoint:
                // * We start off with 'out' containing our initial token, and `temp` empty
                // * If we are able to break any tokens in `out`, then `out` will have
                //   at least one more element than 'temp', so we will try to break tokens
                //   again.
                // * If we cannot break any tokens in 'out', we are done
                loop {
                    let mut temp = SmallVec::<[_; 2]>::new();
                    let mut changed = false;

                    for token in out.into_iter() {
                        if let Some((first, second)) = token.kind.break_two_token_op() {
                            temp.push(Token::new(first, DUMMY_SP));
                            temp.push(Token::new(second, DUMMY_SP));
                            changed = true;
                        } else {
                            temp.push(token);
                        }
                    }
                    out = temp;
                    if !changed {
                        break;
                    }
                }
                token_trees = out.into_iter().map(|t| TokenTree::Token(t)).collect();
                if token_trees.len() != 1 {
                    debug!("break_tokens: broke {:?} to {:?}", tree, token_trees);
                }
            } else {
                token_trees = SmallVec::new();
                token_trees.push(tree);
            }
            token_trees.into_iter()
        }

        let mut t1 = self.trees().filter(semantic_tree).flat_map(break_tokens);
        let mut t2 = other.trees().filter(semantic_tree).flat_map(break_tokens);
        for (t1, t2) in t1.by_ref().zip(t2.by_ref()) {
            if !t1.probably_equal_for_proc_macro(&t2) {
                return false;
            }
        }
        t1.next().is_none() && t2.next().is_none()
    }

    pub fn map_enumerated<F: FnMut(usize, TokenTree) -> TokenTree>(self, mut f: F) -> TokenStream {
        TokenStream(Lrc::new(
            self.0
                .iter()
                .enumerate()
                .map(|(i, (tree, is_joint))| (f(i, tree.clone()), *is_joint))
                .collect(),
        ))
    }

    pub fn map<F: FnMut(TokenTree) -> TokenTree>(self, mut f: F) -> TokenStream {
        TokenStream(Lrc::new(
            self.0.iter().map(|(tree, is_joint)| (f(tree.clone()), *is_joint)).collect(),
        ))
    }
}

// 99.5%+ of the time we have 1 or 2 elements in this vector.
#[derive(Clone)]
pub struct TokenStreamBuilder(SmallVec<[TokenStream; 2]>);

impl TokenStreamBuilder {
    pub fn new() -> TokenStreamBuilder {
        TokenStreamBuilder(SmallVec::new())
    }

    pub fn push<T: Into<TokenStream>>(&mut self, stream: T) {
        let mut stream = stream.into();

        // If `self` is not empty and the last tree within the last stream is a
        // token tree marked with `Joint`...
        if let Some(TokenStream(ref mut last_stream_lrc)) = self.0.last_mut() {
            if let Some((TokenTree::Token(last_token), Joint)) = last_stream_lrc.last() {
                // ...and `stream` is not empty and the first tree within it is
                // a token tree...
                let TokenStream(ref mut stream_lrc) = stream;
                if let Some((TokenTree::Token(token), is_joint)) = stream_lrc.first() {
                    // ...and the two tokens can be glued together...
                    if let Some(glued_tok) = last_token.glue(&token) {
                        // ...then do so, by overwriting the last token
                        // tree in `self` and removing the first token tree
                        // from `stream`. This requires using `make_mut()`
                        // on the last stream in `self` and on `stream`,
                        // and in practice this doesn't cause cloning 99.9%
                        // of the time.

                        // Overwrite the last token tree with the merged
                        // token.
                        let last_vec_mut = Lrc::make_mut(last_stream_lrc);
                        *last_vec_mut.last_mut().unwrap() =
                            (TokenTree::Token(glued_tok), *is_joint);

                        // Remove the first token tree from `stream`. (This
                        // is almost always the only tree in `stream`.)
                        let stream_vec_mut = Lrc::make_mut(stream_lrc);
                        stream_vec_mut.remove(0);

                        // Don't push `stream` if it's empty -- that could
                        // block subsequent token gluing, by getting
                        // between two token trees that should be glued
                        // together.
                        if !stream.is_empty() {
                            self.0.push(stream);
                        }
                        return;
                    }
                }
            }
        }
        self.0.push(stream);
    }

    pub fn build(self) -> TokenStream {
        TokenStream::from_streams(self.0)
    }
}

#[derive(Clone)]
pub struct Cursor {
    pub stream: TokenStream,
    index: usize,
}

impl Iterator for Cursor {
    type Item = TokenTree;

    fn next(&mut self) -> Option<TokenTree> {
        self.next_with_joint().map(|(tree, _)| tree)
    }
}

impl Cursor {
    fn new(stream: TokenStream) -> Self {
        Cursor { stream, index: 0 }
    }

    pub fn next_with_joint(&mut self) -> Option<TreeAndJoint> {
        if self.index < self.stream.len() {
            self.index += 1;
            Some(self.stream.0[self.index - 1].clone())
        } else {
            None
        }
    }

    pub fn append(&mut self, new_stream: TokenStream) {
        if new_stream.is_empty() {
            return;
        }
        let index = self.index;
        let stream = mem::take(&mut self.stream);
        *self = TokenStream::from_streams(smallvec![stream, new_stream]).into_trees();
        self.index = index;
    }

    pub fn look_ahead(&self, n: usize) -> Option<TokenTree> {
        self.stream.0[self.index..].get(n).map(|(tree, _)| tree.clone())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, RustcEncodable, RustcDecodable, HashStable_Generic)]
pub struct DelimSpan {
    pub open: Span,
    pub close: Span,
}

impl DelimSpan {
    pub fn from_single(sp: Span) -> Self {
        DelimSpan { open: sp, close: sp }
    }

    pub fn from_pair(open: Span, close: Span) -> Self {
        DelimSpan { open, close }
    }

    pub fn dummy() -> Self {
        Self::from_single(DUMMY_SP)
    }

    pub fn entire(self) -> Span {
        self.open.with_hi(self.close.hi())
    }
}
