use super::ty::{AllowPlus, RecoverQPath};
use super::{Parser, TokenType};
use crate::maybe_whole;
use rustc_ast::ast::{self, AngleBracketedArg, AngleBracketedArgs, GenericArg, ParenthesizedArgs};
use rustc_ast::ast::{AnonConst, AssocTyConstraint, AssocTyConstraintKind, BlockCheckMode};
use rustc_ast::ast::{Path, PathSegment, QSelf};
use rustc_ast::ptr::P;
use rustc_ast::token::{self, Token};
use rustc_errors::{pluralize, Applicability, PResult};
use rustc_span::source_map::{BytePos, Span};
use rustc_span::symbol::{kw, sym, Ident};

use log::debug;
use std::mem;

/// Specifies how to parse a path.
#[derive(Copy, Clone, PartialEq)]
pub enum PathStyle {
    /// In some contexts, notably in expressions, paths with generic arguments are ambiguous
    /// with something else. For example, in expressions `segment < ....` can be interpreted
    /// as a comparison and `segment ( ....` can be interpreted as a function call.
    /// In all such contexts the non-path interpretation is preferred by default for practical
    /// reasons, but the path interpretation can be forced by the disambiguator `::`, e.g.
    /// `x<y>` - comparisons, `x::<y>` - unambiguously a path.
    Expr,
    /// In other contexts, notably in types, no ambiguity exists and paths can be written
    /// without the disambiguator, e.g., `x<y>` - unambiguously a path.
    /// Paths with disambiguators are still accepted, `x::<Y>` - unambiguously a path too.
    Type,
    /// A path with generic arguments disallowed, e.g., `foo::bar::Baz`, used in imports,
    /// visibilities or attributes.
    /// Technically, this variant is unnecessary and e.g., `Expr` can be used instead
    /// (paths in "mod" contexts have to be checked later for absence of generic arguments
    /// anyway, due to macros), but it is used to avoid weird suggestions about expected
    /// tokens when something goes wrong.
    Mod,
}

impl<'a> Parser<'a> {
    /// Parses a qualified path.
    /// Assumes that the leading `<` has been parsed already.
    ///
    /// `qualified_path = <type [as trait_ref]>::path`
    ///
    /// # Examples
    /// `<T>::default`
    /// `<T as U>::a`
    /// `<T as U>::F::a<S>` (without disambiguator)
    /// `<T as U>::F::a::<S>` (with disambiguator)
    pub(super) fn parse_qpath(&mut self, style: PathStyle) -> PResult<'a, (QSelf, Path)> {
        let lo = self.prev_token.span;
        let ty = self.parse_ty()?;

        // `path` will contain the prefix of the path up to the `>`,
        // if any (e.g., `U` in the `<T as U>::*` examples
        // above). `path_span` has the span of that path, or an empty
        // span in the case of something like `<T>::Bar`.
        let (mut path, path_span);
        if self.eat_keyword(kw::As) {
            let path_lo = self.token.span;
            path = self.parse_path(PathStyle::Type)?;
            path_span = path_lo.to(self.prev_token.span);
        } else {
            path_span = self.token.span.to(self.token.span);
            path = ast::Path { segments: Vec::new(), span: path_span };
        }

        // See doc comment for `unmatched_angle_bracket_count`.
        self.expect(&token::Gt)?;
        if self.unmatched_angle_bracket_count > 0 {
            self.unmatched_angle_bracket_count -= 1;
            debug!("parse_qpath: (decrement) count={:?}", self.unmatched_angle_bracket_count);
        }

        if !self.recover_colon_before_qpath_proj() {
            self.expect(&token::ModSep)?;
        }

        let qself = QSelf { ty, path_span, position: path.segments.len() };
        self.parse_path_segments(&mut path.segments, style)?;

        Ok((qself, Path { segments: path.segments, span: lo.to(self.prev_token.span) }))
    }

    /// Recover from an invalid single colon, when the user likely meant a qualified path.
    /// We avoid emitting this if not followed by an identifier, as our assumption that the user
    /// intended this to be a qualified path may not be correct.
    ///
    /// ```ignore (diagnostics)
    /// <Bar as Baz<T>>:Qux
    ///                ^ help: use double colon
    /// ```
    fn recover_colon_before_qpath_proj(&mut self) -> bool {
        if self.token.kind != token::Colon
            || self.look_ahead(1, |t| !t.is_ident() || t.is_reserved_ident())
        {
            return false;
        }

        self.bump(); // colon

        self.diagnostic()
            .struct_span_err(
                self.prev_token.span,
                "found single colon before projection in qualified path",
            )
            .span_suggestion(
                self.prev_token.span,
                "use double colon",
                "::".to_string(),
                Applicability::MachineApplicable,
            )
            .emit();

        true
    }

    /// Parses simple paths.
    ///
    /// `path = [::] segment+`
    /// `segment = ident | ident[::]<args> | ident[::](args) [-> type]`
    ///
    /// # Examples
    /// `a::b::C<D>` (without disambiguator)
    /// `a::b::C::<D>` (with disambiguator)
    /// `Fn(Args)` (without disambiguator)
    /// `Fn::(Args)` (with disambiguator)
    pub fn parse_path(&mut self, style: PathStyle) -> PResult<'a, Path> {
        maybe_whole!(self, NtPath, |path| {
            if style == PathStyle::Mod && path.segments.iter().any(|segment| segment.args.is_some())
            {
                self.struct_span_err(path.span, "unexpected generic arguments in path").emit();
            }
            path
        });

        let lo = self.token.span;
        let mut segments = Vec::new();
        let mod_sep_ctxt = self.token.span.ctxt();
        if self.eat(&token::ModSep) {
            segments.push(PathSegment::path_root(lo.shrink_to_lo().with_ctxt(mod_sep_ctxt)));
        }
        self.parse_path_segments(&mut segments, style)?;

        Ok(Path { segments, span: lo.to(self.prev_token.span) })
    }

    pub(super) fn parse_path_segments(
        &mut self,
        segments: &mut Vec<PathSegment>,
        style: PathStyle,
    ) -> PResult<'a, ()> {
        loop {
            let segment = self.parse_path_segment(style)?;
            if style == PathStyle::Expr {
                // In order to check for trailing angle brackets, we must have finished
                // recursing (`parse_path_segment` can indirectly call this function),
                // that is, the next token must be the highlighted part of the below example:
                //
                // `Foo::<Bar as Baz<T>>::Qux`
                //                      ^ here
                //
                // As opposed to the below highlight (if we had only finished the first
                // recursion):
                //
                // `Foo::<Bar as Baz<T>>::Qux`
                //                     ^ here
                //
                // `PathStyle::Expr` is only provided at the root invocation and never in
                // `parse_path_segment` to recurse and therefore can be checked to maintain
                // this invariant.
                self.check_trailing_angle_brackets(&segment, token::ModSep);
            }
            segments.push(segment);

            if self.is_import_coupler() || !self.eat(&token::ModSep) {
                return Ok(());
            }
        }
    }

    pub(super) fn parse_path_segment(&mut self, style: PathStyle) -> PResult<'a, PathSegment> {
        let ident = self.parse_path_segment_ident()?;

        let is_args_start = |token: &Token| match token.kind {
            token::Lt
            | token::BinOp(token::Shl)
            | token::OpenDelim(token::Paren)
            | token::LArrow => true,
            _ => false,
        };
        let check_args_start = |this: &mut Self| {
            this.expected_tokens.extend_from_slice(&[
                TokenType::Token(token::Lt),
                TokenType::Token(token::OpenDelim(token::Paren)),
            ]);
            is_args_start(&this.token)
        };

        Ok(
            if style == PathStyle::Type && check_args_start(self)
                || style != PathStyle::Mod
                    && self.check(&token::ModSep)
                    && self.look_ahead(1, |t| is_args_start(t))
            {
                // We use `style == PathStyle::Expr` to check if this is in a recursion or not. If
                // it isn't, then we reset the unmatched angle bracket count as we're about to start
                // parsing a new path.
                if style == PathStyle::Expr {
                    self.unmatched_angle_bracket_count = 0;
                    self.max_angle_bracket_count = 0;
                }

                // Generic arguments are found - `<`, `(`, `::<` or `::(`.
                self.eat(&token::ModSep);
                let lo = self.token.span;
                let args = if self.eat_lt() {
                    // `<'a, T, A = U>`
                    let args =
                        self.parse_angle_args_with_leading_angle_bracket_recovery(style, lo)?;
                    self.expect_gt()?;
                    let span = lo.to(self.prev_token.span);
                    AngleBracketedArgs { args, span }.into()
                } else {
                    // `(T, U) -> R`
                    let (inputs, _) = self.parse_paren_comma_seq(|p| p.parse_ty())?;
                    let span = ident.span.to(self.prev_token.span);
                    let output = self.parse_ret_ty(AllowPlus::No, RecoverQPath::No)?;
                    ParenthesizedArgs { inputs, output, span }.into()
                };

                PathSegment { ident, args, id: ast::DUMMY_NODE_ID }
            } else {
                // Generic arguments are not found.
                PathSegment::from_ident(ident)
            },
        )
    }

    pub(super) fn parse_path_segment_ident(&mut self) -> PResult<'a, Ident> {
        match self.token.ident() {
            Some((ident, false)) if ident.is_path_segment_keyword() => {
                self.bump();
                Ok(ident)
            }
            _ => self.parse_ident(),
        }
    }

    /// Parses generic args (within a path segment) with recovery for extra leading angle brackets.
    /// For the purposes of understanding the parsing logic of generic arguments, this function
    /// can be thought of being the same as just calling `self.parse_angle_args()` if the source
    /// had the correct amount of leading angle brackets.
    ///
    /// ```ignore (diagnostics)
    /// bar::<<<<T as Foo>::Output>();
    ///      ^^ help: remove extra angle brackets
    /// ```
    fn parse_angle_args_with_leading_angle_bracket_recovery(
        &mut self,
        style: PathStyle,
        lo: Span,
    ) -> PResult<'a, Vec<AngleBracketedArg>> {
        // We need to detect whether there are extra leading left angle brackets and produce an
        // appropriate error and suggestion. This cannot be implemented by looking ahead at
        // upcoming tokens for a matching `>` character - if there are unmatched `<` tokens
        // then there won't be matching `>` tokens to find.
        //
        // To explain how this detection works, consider the following example:
        //
        // ```ignore (diagnostics)
        // bar::<<<<T as Foo>::Output>();
        //      ^^ help: remove extra angle brackets
        // ```
        //
        // Parsing of the left angle brackets starts in this function. We start by parsing the
        // `<` token (incrementing the counter of unmatched angle brackets on `Parser` via
        // `eat_lt`):
        //
        // *Upcoming tokens:* `<<<<T as Foo>::Output>;`
        // *Unmatched count:* 1
        // *`parse_path_segment` calls deep:* 0
        //
        // This has the effect of recursing as this function is called if a `<` character
        // is found within the expected generic arguments:
        //
        // *Upcoming tokens:* `<<<T as Foo>::Output>;`
        // *Unmatched count:* 2
        // *`parse_path_segment` calls deep:* 1
        //
        // Eventually we will have recursed until having consumed all of the `<` tokens and
        // this will be reflected in the count:
        //
        // *Upcoming tokens:* `T as Foo>::Output>;`
        // *Unmatched count:* 4
        // `parse_path_segment` calls deep:* 3
        //
        // The parser will continue until reaching the first `>` - this will decrement the
        // unmatched angle bracket count and return to the parent invocation of this function
        // having succeeded in parsing:
        //
        // *Upcoming tokens:* `::Output>;`
        // *Unmatched count:* 3
        // *`parse_path_segment` calls deep:* 2
        //
        // This will continue until the next `>` character which will also return successfully
        // to the parent invocation of this function and decrement the count:
        //
        // *Upcoming tokens:* `;`
        // *Unmatched count:* 2
        // *`parse_path_segment` calls deep:* 1
        //
        // At this point, this function will expect to find another matching `>` character but
        // won't be able to and will return an error. This will continue all the way up the
        // call stack until the first invocation:
        //
        // *Upcoming tokens:* `;`
        // *Unmatched count:* 2
        // *`parse_path_segment` calls deep:* 0
        //
        // In doing this, we have managed to work out how many unmatched leading left angle
        // brackets there are, but we cannot recover as the unmatched angle brackets have
        // already been consumed. To remedy this, we keep a snapshot of the parser state
        // before we do the above. We can then inspect whether we ended up with a parsing error
        // and unmatched left angle brackets and if so, restore the parser state before we
        // consumed any `<` characters to emit an error and consume the erroneous tokens to
        // recover by attempting to parse again.
        //
        // In practice, the recursion of this function is indirect and there will be other
        // locations that consume some `<` characters - as long as we update the count when
        // this happens, it isn't an issue.

        let is_first_invocation = style == PathStyle::Expr;
        // Take a snapshot before attempting to parse - we can restore this later.
        let snapshot = if is_first_invocation { Some(self.clone()) } else { None };

        debug!("parse_generic_args_with_leading_angle_bracket_recovery: (snapshotting)");
        match self.parse_angle_args() {
            Ok(args) => Ok(args),
            Err(ref mut e) if is_first_invocation && self.unmatched_angle_bracket_count > 0 => {
                // Cancel error from being unable to find `>`. We know the error
                // must have been this due to a non-zero unmatched angle bracket
                // count.
                e.cancel();

                // Swap `self` with our backup of the parser state before attempting to parse
                // generic arguments.
                let snapshot = mem::replace(self, snapshot.unwrap());

                debug!(
                    "parse_generic_args_with_leading_angle_bracket_recovery: (snapshot failure) \
                     snapshot.count={:?}",
                    snapshot.unmatched_angle_bracket_count,
                );

                // Eat the unmatched angle brackets.
                for _ in 0..snapshot.unmatched_angle_bracket_count {
                    self.eat_lt();
                }

                // Make a span over ${unmatched angle bracket count} characters.
                let span = lo.with_hi(lo.lo() + BytePos(snapshot.unmatched_angle_bracket_count));
                self.struct_span_err(
                    span,
                    &format!(
                        "unmatched angle bracket{}",
                        pluralize!(snapshot.unmatched_angle_bracket_count)
                    ),
                )
                .span_suggestion(
                    span,
                    &format!(
                        "remove extra angle bracket{}",
                        pluralize!(snapshot.unmatched_angle_bracket_count)
                    ),
                    String::new(),
                    Applicability::MachineApplicable,
                )
                .emit();

                // Try again without unmatched angle bracket characters.
                self.parse_angle_args()
            }
            Err(e) => Err(e),
        }
    }

    /// Parses (possibly empty) list of generic arguments / associated item constraints,
    /// possibly including trailing comma.
    fn parse_angle_args(&mut self) -> PResult<'a, Vec<AngleBracketedArg>> {
        let mut args = Vec::new();
        while let Some(arg) = self.parse_angle_arg()? {
            args.push(arg);
            if !self.eat(&token::Comma) {
                break;
            }
        }
        Ok(args)
    }

    /// Parses a single argument in the angle arguments `<...>` of a path segment.
    fn parse_angle_arg(&mut self) -> PResult<'a, Option<AngleBracketedArg>> {
        if self.check_ident() && self.look_ahead(1, |t| matches!(t.kind, token::Eq | token::Colon))
        {
            // Parse associated type constraint.
            let lo = self.token.span;
            let ident = self.parse_ident()?;
            let kind = if self.eat(&token::Eq) {
                let ty = self.parse_assoc_equality_term(ident, self.prev_token.span)?;
                AssocTyConstraintKind::Equality { ty }
            } else if self.eat(&token::Colon) {
                let bounds = self.parse_generic_bounds(Some(self.prev_token.span))?;
                AssocTyConstraintKind::Bound { bounds }
            } else {
                unreachable!();
            };

            let span = lo.to(self.prev_token.span);

            // Gate associated type bounds, e.g., `Iterator<Item: Ord>`.
            if let AssocTyConstraintKind::Bound { .. } = kind {
                self.sess.gated_spans.gate(sym::associated_type_bounds, span);
            }

            let constraint = AssocTyConstraint { id: ast::DUMMY_NODE_ID, ident, kind, span };
            Ok(Some(AngleBracketedArg::Constraint(constraint)))
        } else {
            Ok(self.parse_generic_arg()?.map(AngleBracketedArg::Arg))
        }
    }

    /// Parse the term to the right of an associated item equality constraint.
    /// That is, parse `<term>` in `Item = <term>`.
    /// Right now, this only admits types in `<term>`.
    fn parse_assoc_equality_term(&mut self, ident: Ident, eq: Span) -> PResult<'a, P<ast::Ty>> {
        let arg = self.parse_generic_arg()?;
        let span = ident.span.to(self.prev_token.span);
        match arg {
            Some(GenericArg::Type(ty)) => return Ok(ty),
            Some(GenericArg::Const(expr)) => {
                self.struct_span_err(span, "cannot constrain an associated constant to a value")
                    .span_label(ident.span, "this associated constant...")
                    .span_label(expr.value.span, "...cannot be constrained to this value")
                    .emit();
            }
            Some(GenericArg::Lifetime(lt)) => {
                self.struct_span_err(span, "associated lifetimes are not supported")
                    .span_label(lt.ident.span, "the lifetime is given here")
                    .help("if you meant to specify a trait object, write `dyn Trait + 'lifetime`")
                    .emit();
            }
            None => {
                let after_eq = eq.shrink_to_hi();
                let before_next = self.token.span.shrink_to_lo();
                self.struct_span_err(after_eq.to(before_next), "missing type to the right of `=`")
                    .span_suggestion(
                        self.sess.source_map().next_point(eq).to(before_next),
                        "to constrain the associated type, add a type after `=`",
                        " TheType".to_string(),
                        Applicability::HasPlaceholders,
                    )
                    .span_suggestion(
                        eq.to(before_next),
                        &format!("remove the `=` if `{}` is a type", ident),
                        String::new(),
                        Applicability::MaybeIncorrect,
                    )
                    .emit();
            }
        }
        Ok(self.mk_ty(span, ast::TyKind::Err))
    }

    /// Parse a generic argument in a path segment.
    /// This does not include constraints, e.g., `Item = u8`, which is handled in `parse_angle_arg`.
    fn parse_generic_arg(&mut self) -> PResult<'a, Option<GenericArg>> {
        let arg = if self.check_lifetime() && self.look_ahead(1, |t| !t.is_like_plus()) {
            // Parse lifetime argument.
            GenericArg::Lifetime(self.expect_lifetime())
        } else if self.check_const_arg() {
            // Parse const argument.
            let expr = if let token::OpenDelim(token::Brace) = self.token.kind {
                self.parse_block_expr(
                    None,
                    self.token.span,
                    BlockCheckMode::Default,
                    ast::AttrVec::new(),
                )?
            } else if self.token.is_ident() {
                // FIXME(const_generics): to distinguish between idents for types and consts,
                // we should introduce a GenericArg::Ident in the AST and distinguish when
                // lowering to the HIR. For now, idents for const args are not permitted.
                if self.token.is_bool_lit() {
                    self.parse_literal_maybe_minus()?
                } else {
                    let span = self.token.span;
                    let msg = "identifiers may currently not be used for const generics";
                    self.struct_span_err(span, msg).emit();
                    let block = self.mk_block_err(span);
                    self.mk_expr(span, ast::ExprKind::Block(block, None), ast::AttrVec::new())
                }
            } else {
                self.parse_literal_maybe_minus()?
            };
            GenericArg::Const(AnonConst { id: ast::DUMMY_NODE_ID, value: expr })
        } else if self.check_type() {
            // Parse type argument.
            GenericArg::Type(self.parse_ty()?)
        } else {
            return Ok(None);
        };
        Ok(Some(arg))
    }
}
