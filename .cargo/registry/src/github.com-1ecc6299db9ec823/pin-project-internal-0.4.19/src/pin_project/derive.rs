use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, quote_spanned, ToTokens};
use std::fmt;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    visit_mut::VisitMut,
    *,
};

use super::PIN;
use crate::utils::{
    determine_lifetime_name, determine_visibility, insert_lifetime_and_bound, ParseBufferExt,
    ProjKind, ReplaceReceiver, SliceExt, Variants,
};

pub(super) fn parse_derive(input: TokenStream) -> Result<TokenStream> {
    let mut input = syn::parse2(input)?;
    let DeriveInput { attrs, vis, ident, generics, data } = &mut input;
    let mut cx = Context::new(attrs, vis, ident, generics)?;
    let packed_check;

    let (mut items, scoped_items) = match data {
        Data::Struct(data) => {
            // Do this first for a better error message.
            packed_check = Some(cx.ensure_not_packed(&data.fields)?);
            cx.parse_struct(data)?
        }
        Data::Enum(data) => {
            // We don't need to check for `#[repr(packed)]`,
            // since it does not apply to enums.
            packed_check = None;
            cx.parse_enum(data)?
        }
        Data::Union(_) => {
            return Err(error!(
                input,
                "#[pin_project] attribute may only be used on structs or enums"
            ));
        }
    };

    let unpin_impl = cx.make_unpin_impl();
    let drop_impl = cx.make_drop_impl();
    let dummy_const = format_ident!("__SCOPE_{}", ident);
    items.extend(quote! {
        // All items except projected types are generated inside a `const` scope.
        // This makes it impossible for user code to refer to these types.
        // However, this prevents Rustdoc from displaying docs for any
        // of our types. In particular, users cannot see the
        // automatically generated `Unpin` impl for the '__UnpinStruct' types
        //
        // Previously, we provided a flag to correctly document the
        // automatically generated `Unpin` impl by using def-site hygiene,
        // but it is now removed.
        //
        // Refs:
        // * https://github.com/rust-lang/rust/issues/63281
        // * https://github.com/taiki-e/pin-project/pull/53#issuecomment-525906867
        // * https://github.com/taiki-e/pin-project/pull/70
        #[doc(hidden)]
        #[allow(non_upper_case_globals)]
        #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
        #[allow(clippy::used_underscore_binding)]
        const #dummy_const: () = {
            #scoped_items
            #unpin_impl
            #drop_impl
            #packed_check
        };
    });
    Ok(items)
}

fn validate_struct(ident: &Ident, fields: &Fields) -> Result<()> {
    if fields.is_empty() {
        let msg = "#[pin_project] attribute may not be used on structs with zero fields";
        if let Fields::Unit = fields { Err(error!(ident, msg)) } else { Err(error!(fields, msg)) }
    } else {
        Ok(())
    }
}

fn validate_enum(brace_token: token::Brace, variants: &Variants) -> Result<()> {
    if variants.is_empty() {
        return Err(Error::new(
            brace_token.span,
            "#[pin_project] attribute may not be used on enums without variants",
        ));
    }
    let has_field = variants.iter().try_fold(false, |has_field, v| {
        if let Some((_, e)) = &v.discriminant {
            Err(error!(e, "#[pin_project] attribute may not be used on enums with discriminants"))
        } else if let Some(attr) = v.attrs.find(PIN) {
            Err(error!(attr, "#[pin] attribute may only be used on fields of structs or variants"))
        } else if v.fields.is_empty() {
            Ok(has_field)
        } else {
            Ok(true)
        }
    })?;
    if has_field {
        Ok(())
    } else {
        Err(error!(variants, "#[pin_project] attribute may not be used on enums with zero fields"))
    }
}

struct Args {
    /// `PinnedDrop` argument.
    pinned_drop: Option<Span>,
    /// `Replace` argument.
    replace: Option<Span>,
    /// `UnsafeUnpin` or `!Unpin` argument.
    unpin_impl: UnpinImpl,
    /// `project = <ident>`.
    project: Option<Ident>,
    /// `project_ref = <ident>`.
    project_ref: Option<Ident>,
    /// `project_replace = <ident>`.
    project_replace: Option<Ident>,
}

const DUPLICATE_PIN: &str = "duplicate #[pin] attribute";

impl Args {
    fn get(attrs: &[Attribute]) -> Result<Self> {
        // `(__private(<args>))` -> `<args>`
        struct Input(Option<TokenStream>);

        impl Parse for Input {
            fn parse(input: ParseStream<'_>) -> Result<Self> {
                Ok(Self((|| {
                    let content = input.parenthesized().ok()?;
                    let private = content.parse::<Ident>().ok()?;
                    if private == "__private" {
                        content.parenthesized().ok()?.parse::<TokenStream>().ok()
                    } else {
                        None
                    }
                })()))
            }
        }

        if let Some(attr) = attrs.find("pin_project") {
            return Err(error!(attr, "duplicate #[pin_project] attribute"));
        }

        let mut attrs = attrs.iter().filter(|attr| attr.path.is_ident(PIN));

        let prev = if let Some(attr) = attrs.next() {
            (attr, syn::parse2::<Input>(attr.tokens.clone()).unwrap().0)
        } else {
            // This only fails if another macro removes `#[pin]`.
            return Err(error!(TokenStream::new(), "#[pin_project] attribute has been removed"));
        };

        if let Some(attr) = attrs.next() {
            let (prev_attr, prev_res) = &prev;
            // As the `#[pin]` attribute generated by `#[pin_project]`
            // has the same span as `#[pin_project]`, it is possible
            // that a useless error message will be generated.
            let res = syn::parse2::<Input>(attr.tokens.clone()).unwrap().0;
            let span = match (&prev_res, res) {
                (Some(_), _) => attr,
                (_, Some(_)) => prev_attr,
                (None, None) => prev_attr,
            };
            Err(error!(span, DUPLICATE_PIN))
        } else {
            // This only fails if another macro removes `#[pin]` and inserts own `#[pin]`.
            syn::parse2(prev.1.unwrap())
        }
    }
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        mod kw {
            syn::custom_keyword!(Unpin);
        }

        // Parses `= <value>` in `<name> = <value>` and returns value and span of name-value pair.
        fn parse_value(
            input: ParseStream<'_>,
            name: &Ident,
            has_prev: bool,
        ) -> Result<(Ident, TokenStream)> {
            if input.is_empty() {
                return Err(error!(name, "expected `{0} = <identifier>`, found `{0}`", name));
            }
            let eq_token: token::Eq = input.parse()?;
            if input.is_empty() {
                let span = quote!(#name #eq_token);
                return Err(error!(span, "expected `{0} = <identifier>`, found `{0} =`", name));
            }
            let value: Ident = input.parse()?;
            let span = quote!(#name #value);
            if has_prev {
                Err(error!(span, "duplicate `{}` argument", name))
            } else {
                Ok((value, span))
            }
        }

        // Replace `prev` with `new`. Returns `Err` if `prev` is `Some`.
        fn update_value<T>(
            prev: &mut Option<T>,
            new: T,
            token: &(impl ToTokens + fmt::Display),
        ) -> Result<()> {
            if prev.replace(new).is_some() {
                Err(error!(token, "duplicate `{}` argument", token.to_string().replace(' ', "")))
            } else {
                Ok(())
            }
        }

        let mut pinned_drop = None;
        let mut replace = None;
        let mut unsafe_unpin = None;
        let mut not_unpin = None;
        let mut project = None;
        let mut project_ref = None;
        let mut project_replace = None;
        while !input.is_empty() {
            if input.peek(token::Bang) {
                let bang: token::Bang = input.parse()?;
                if input.is_empty() {
                    return Err(error!(bang, "expected `!Unpin`, found `!`"));
                }
                let unpin: kw::Unpin = input.parse()?;
                let span = quote!(#bang #unpin);
                update_value(&mut not_unpin, span.span(), &span)?;
            } else {
                let token = input.parse::<Ident>()?;
                match &*token.to_string() {
                    "PinnedDrop" => {
                        update_value(&mut pinned_drop, token.span(), &token)?;
                    }
                    "Replace" => {
                        update_value(&mut replace, token.span(), &token)?;
                    }
                    "UnsafeUnpin" => {
                        update_value(&mut unsafe_unpin, token.span(), &token)?;
                    }
                    "project" => {
                        project = Some(parse_value(input, &token, project.is_some())?.0);
                    }
                    "project_ref" => {
                        project_ref = Some(parse_value(input, &token, project_ref.is_some())?.0);
                    }
                    "project_replace" => {
                        project_replace =
                            Some(parse_value(input, &token, project_replace.is_some())?);
                    }
                    _ => return Err(error!(token, "unexpected argument: {}", token)),
                }
            }

            if input.is_empty() {
                break;
            }
            let _: token::Comma = input.parse()?;
        }

        if let (Some((_, span)), None) = (&project_replace, replace) {
            return Err(error!(
                span,
                "`project_replace` argument can only be used together with `Replace` argument",
            ));
        }
        if let (Some(span), Some(_)) = (pinned_drop, replace) {
            return Err(Error::new(
                span,
                "arguments `PinnedDrop` and `Replace` are mutually exclusive",
            ));
        }
        let unpin_impl = match (unsafe_unpin, not_unpin) {
            (None, None) => UnpinImpl::Default,
            (Some(span), None) => UnpinImpl::Unsafe(span),
            (None, Some(span)) => UnpinImpl::Negative(span),
            (Some(span), Some(_)) => {
                return Err(Error::new(
                    span,
                    "arguments `UnsafeUnpin` and `!Unpin` are mutually exclusive",
                ));
            }
        };

        Ok(Self {
            pinned_drop,
            replace,
            unpin_impl,
            project,
            project_ref,
            project_replace: project_replace.map(|(i, _)| i),
        })
    }
}

struct OriginalType<'a> {
    /// Attributes of the original type.
    attrs: &'a [Attribute],
    /// Visibility of the original type.
    vis: &'a Visibility,
    /// Name of the original type.
    ident: &'a Ident,
    /// Generics of the original type.
    generics: &'a Generics,
}

struct ProjectedType {
    /// Visibility of the projected types.
    vis: Visibility,
    /// Name of the projected type returned by `project` method.
    mut_ident: Ident,
    /// Name of the projected type returned by `project_ref` method.
    ref_ident: Ident,
    /// Name of the projected type returned by `project_replace` method.
    own_ident: Ident,
    /// Lifetime on the generated projected types.
    lifetime: Lifetime,
    /// Generics of the projected types.
    generics: Generics,
    /// `where` clause of the projected types. This has an additional
    /// bound generated by `insert_lifetime_and_bound`
    where_clause: WhereClause,
}

struct ProjectedVariants {
    proj_variants: TokenStream,
    proj_ref_variants: TokenStream,
    proj_own_variants: TokenStream,
    proj_arms: TokenStream,
    proj_ref_arms: TokenStream,
    proj_own_arms: TokenStream,
}

#[derive(Default)]
struct ProjectedFields {
    proj_pat: TokenStream,
    proj_body: TokenStream,
    proj_fields: TokenStream,
    proj_ref_fields: TokenStream,
    proj_own_fields: TokenStream,
    proj_move: TokenStream,
    proj_drop: Vec<Ident>,
}

struct Context<'a> {
    /// The original type.
    orig: OriginalType<'a>,
    /// The projected types.
    proj: ProjectedType,
    /// Types of the pinned fields.
    pinned_fields: Vec<Type>,

    /// `PinnedDrop` argument.
    pinned_drop: Option<Span>,
    /// `Replace` argument (requires Sized bound)
    replace: Option<Span>,
    /// `UnsafeUnpin` or `!Unpin` argument.
    unpin_impl: UnpinImpl,
    /// `project` argument.
    project: bool,
    /// `project_ref` argument.
    project_ref: bool,
    /// `project_replace` argument.
    project_replace: bool,
}

#[derive(Clone, Copy)]
enum UnpinImpl {
    Default,
    /// `UnsafeUnpin`.
    Unsafe(Span),
    /// `!Unpin`.
    Negative(Span),
}

impl<'a> Context<'a> {
    fn new(
        attrs: &'a [Attribute],
        vis: &'a Visibility,
        ident: &'a Ident,
        generics: &'a mut Generics,
    ) -> Result<Self> {
        let Args { pinned_drop, unpin_impl, replace, project, project_ref, project_replace } =
            Args::get(attrs)?;

        let ty_generics = generics.split_for_impl().1;
        let self_ty = syn::parse_quote!(#ident #ty_generics);
        let mut visitor = ReplaceReceiver(&self_ty);
        visitor.visit_where_clause_mut(generics.make_where_clause());

        let mut lifetime_name = String::from("'pin");
        determine_lifetime_name(&mut lifetime_name, generics);
        let lifetime = Lifetime::new(&lifetime_name, Span::call_site());

        let mut proj_generics = generics.clone();
        let ty_generics = generics.split_for_impl().1;
        let ty_generics_as_generics = syn::parse_quote!(#ty_generics);
        let pred = insert_lifetime_and_bound(
            &mut proj_generics,
            lifetime.clone(),
            &ty_generics_as_generics,
            ident,
        );
        let mut where_clause = generics.clone().make_where_clause().clone();
        where_clause.predicates.push(pred);

        Ok(Self {
            pinned_drop,
            replace,
            unpin_impl,
            project: project.is_some(),
            project_ref: project_ref.is_some(),
            project_replace: project_replace.is_some(),
            proj: ProjectedType {
                vis: determine_visibility(vis),
                mut_ident: project.unwrap_or_else(|| ProjKind::Mutable.proj_ident(ident)),
                ref_ident: project_ref.unwrap_or_else(|| ProjKind::Immutable.proj_ident(ident)),
                own_ident: project_replace.unwrap_or_else(|| ProjKind::Owned.proj_ident(ident)),
                lifetime,
                generics: proj_generics,
                where_clause,
            },
            orig: OriginalType { attrs, vis, ident, generics },
            pinned_fields: Vec::new(),
        })
    }

    fn parse_struct(
        &mut self,
        DataStruct { fields, .. }: &DataStruct,
    ) -> Result<(TokenStream, TokenStream)> {
        validate_struct(self.orig.ident, fields)?;

        let ProjectedFields {
            proj_pat,
            proj_body,
            proj_fields,
            proj_ref_fields,
            proj_own_fields,
            proj_move,
            proj_drop,
        } = match fields {
            Fields::Named(fields) => self.visit_named(fields)?,
            Fields::Unnamed(fields) => self.visit_unnamed(fields)?,
            Fields::Unit => unreachable!(),
        };

        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let proj_own_ident = &self.proj.own_ident;
        let vis = &self.proj.vis;
        let mut orig_generics = self.orig.generics.clone();
        let orig_where_clause = orig_generics.where_clause.take();
        let proj_generics = &self.proj.generics;
        let where_clause = &self.proj.where_clause;

        // For tuple structs, we need to generate `(T1, T2) where Foo: Bar`
        // For non-tuple structs, we need to generate `where Foo: Bar { field1: T }`
        let (where_clause_fields, where_clause_ref_fields, where_clause_own_fields) = match fields {
            Fields::Named(_) => (
                quote!(#where_clause #proj_fields),
                quote!(#where_clause #proj_ref_fields),
                quote!(#orig_where_clause #proj_own_fields),
            ),
            Fields::Unnamed(_) => (
                quote!(#proj_fields #where_clause;),
                quote!(#proj_ref_fields #where_clause;),
                quote!(#proj_own_fields #orig_where_clause;),
            ),
            Fields::Unit => unreachable!(),
        };

        // If the user gave it a name, it should appear in the document.
        let doc_attr = quote!(#[doc(hidden)]);
        let doc_proj = if self.project { None } else { Some(&doc_attr) };
        let doc_proj_ref = if self.project_ref { None } else { Some(&doc_attr) };
        let doc_proj_own = if self.project_replace { None } else { Some(&doc_attr) };
        let mut proj_items = quote! {
            #doc_proj
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
            #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
            #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
            #vis struct #proj_ident #proj_generics #where_clause_fields
            #doc_proj_ref
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
            #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
            #vis struct #proj_ref_ident #proj_generics #where_clause_ref_fields
        };
        if self.replace.is_some() {
            // Currently, using quote_spanned here does not seem to have any effect on the
            // diagnostics.
            proj_items.extend(quote! {
                #doc_proj_own
                #[allow(dead_code)] // This lint warns unused fields/variants.
                #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
                #vis struct #proj_own_ident #orig_generics #where_clause_own_fields
            });
        }

        let proj_mut_body = quote! {
            let Self #proj_pat = self.get_unchecked_mut();
            #proj_ident #proj_body
        };
        let proj_ref_body = quote! {
            let Self #proj_pat = self.get_ref();
            #proj_ref_ident #proj_body
        };
        let proj_own_body = quote! {
            let __self_ptr: *mut Self = self.get_unchecked_mut();
            let Self #proj_pat = &mut *__self_ptr;

            // First, extract all the unpinned fields
            let __result = #proj_own_ident #proj_move;

            // Destructors will run in reverse order, so next create a guard to overwrite
            // `self` with the replacement value without calling destructors.
            let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                target: __self_ptr,
                value: ::pin_project::__private::ManuallyDrop::new(__replacement),
            };

            // Now create guards to drop all the pinned fields
            //
            // Due to a compiler bug (https://github.com/rust-lang/rust/issues/47949)
            // this must be in its own scope, or else `__result` will not be dropped
            // if any of the destructors panic.
            {
                #( let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(#proj_drop); )*
            }

            // Finally, return the result
            __result
        };
        let proj_impl = self.make_proj_impl(&proj_mut_body, &proj_ref_body, &proj_own_body);

        Ok((proj_items, proj_impl))
    }

    fn parse_enum(
        &mut self,
        DataEnum { brace_token, variants, .. }: &DataEnum,
    ) -> Result<(TokenStream, TokenStream)> {
        validate_enum(*brace_token, variants)?;

        let ProjectedVariants {
            proj_variants,
            proj_ref_variants,
            proj_own_variants,
            proj_arms,
            proj_ref_arms,
            proj_own_arms,
        } = self.visit_variants(variants)?;

        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let proj_own_ident = &self.proj.own_ident;
        let vis = &self.proj.vis;
        let mut orig_generics = self.orig.generics.clone();
        let orig_where_clause = orig_generics.where_clause.take();
        let proj_generics = &self.proj.generics;
        let where_clause = &self.proj.where_clause;

        // If the user gave it a name, it should appear in the document.
        let doc_attr = quote!(#[doc(hidden)]);
        let doc_proj = if self.project { None } else { Some(&doc_attr) };
        let doc_proj_ref = if self.project_ref { None } else { Some(&doc_attr) };
        let doc_proj_own = if self.project_replace { None } else { Some(&doc_attr) };
        let mut proj_items = quote! {
            #doc_proj
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
            #[allow(clippy::mut_mut)] // This lint warns `&mut &mut <ty>`.
            #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
            #vis enum #proj_ident #proj_generics #where_clause {
                #proj_variants
            }
            #doc_proj_ref
            #[allow(dead_code)] // This lint warns unused fields/variants.
            #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
            #[allow(clippy::type_repetition_in_bounds)] // https://github.com/rust-lang/rust-clippy/issues/4326
            #vis enum #proj_ref_ident #proj_generics #where_clause {
                #proj_ref_variants
            }
        };
        if self.replace.is_some() {
            // Currently, using quote_spanned here does not seem to have any effect on the
            // diagnostics.
            proj_items.extend(quote! {
                #doc_proj_own
                #[allow(dead_code)] // This lint warns unused fields/variants.
                #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/55058
                #vis enum #proj_own_ident #orig_generics #orig_where_clause {
                    #proj_own_variants
                }
            });
        }

        let proj_mut_body = quote! {
            match self.get_unchecked_mut() {
                #proj_arms
            }
        };
        let proj_ref_body = quote! {
            match self.get_ref() {
                #proj_ref_arms
            }
        };
        let proj_own_body = quote! {
            let __self_ptr: *mut Self = self.get_unchecked_mut();
            match &mut *__self_ptr {
                #proj_own_arms
            }
        };
        let proj_impl = self.make_proj_impl(&proj_mut_body, &proj_ref_body, &proj_own_body);

        Ok((proj_items, proj_impl))
    }

    fn visit_variants(&mut self, variants: &Variants) -> Result<ProjectedVariants> {
        let mut proj_variants = TokenStream::new();
        let mut proj_ref_variants = TokenStream::new();
        let mut proj_own_variants = TokenStream::new();
        let mut proj_arms = TokenStream::new();
        let mut proj_ref_arms = TokenStream::new();
        let mut proj_own_arms = TokenStream::new();

        for Variant { ident, fields, .. } in variants {
            let ProjectedFields {
                proj_pat,
                proj_body,
                proj_fields,
                proj_ref_fields,
                proj_own_fields,
                proj_move,
                proj_drop,
            } = match fields {
                Fields::Named(fields) => self.visit_named(fields)?,
                Fields::Unnamed(fields) => self.visit_unnamed(fields)?,
                Fields::Unit => ProjectedFields::default(),
            };

            let orig_ident = self.orig.ident;
            let proj_ident = &self.proj.mut_ident;
            let proj_ref_ident = &self.proj.ref_ident;
            let proj_own_ident = &self.proj.own_ident;
            proj_variants.extend(quote! {
                #ident #proj_fields,
            });
            proj_ref_variants.extend(quote! {
                #ident #proj_ref_fields,
            });
            proj_own_variants.extend(quote! {
                #ident #proj_own_fields,
            });
            proj_arms.extend(quote! {
                #orig_ident::#ident #proj_pat => {
                    #proj_ident::#ident #proj_body
                }
            });
            proj_ref_arms.extend(quote! {
                #orig_ident::#ident #proj_pat => {
                    #proj_ref_ident::#ident #proj_body
                }
            });
            proj_own_arms.extend(quote! {
                #orig_ident::#ident #proj_pat => {
                    // First, extract all the unpinned fields
                    let __result = #proj_own_ident::#ident #proj_move;

                    // Destructors will run in reverse order, so next create a guard to overwrite
                    // `self` with the replacement value without calling destructors.
                    let __guard = ::pin_project::__private::UnsafeOverwriteGuard {
                        target: __self_ptr,
                        value: ::pin_project::__private::ManuallyDrop::new(__replacement),
                    };

                    // Now create guards to drop all the pinned fields
                    //
                    // Due to a compiler bug (https://github.com/rust-lang/rust/issues/47949)
                    // this must be in its own scope, or else `__result` will not be dropped
                    // if any of the destructors panic.
                    {
                        #(
                            let __guard = ::pin_project::__private::UnsafeDropInPlaceGuard(
                                #proj_drop,
                            );
                        )*
                    }

                    // Finally, return the result
                    __result
                }
            });
        }

        Ok(ProjectedVariants {
            proj_variants,
            proj_ref_variants,
            proj_own_variants,
            proj_arms,
            proj_ref_arms,
            proj_own_arms,
        })
    }

    fn visit_named(
        &mut self,
        FieldsNamed { named: fields, .. }: &FieldsNamed,
    ) -> Result<ProjectedFields> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        let mut proj_own_fields = Vec::with_capacity(fields.len());
        let mut proj_move = Vec::with_capacity(fields.len());
        let mut proj_drop = Vec::with_capacity(fields.len());

        for Field { attrs, vis, ident, ty, .. } in fields {
            if attrs.position_exact(PIN)?.is_some() {
                self.pinned_fields.push(ty.clone());
                proj_drop.push(ident.as_ref().cloned().unwrap());

                let lifetime = &self.proj.lifetime;
                proj_fields
                    .push(quote!(#vis #ident: ::pin_project::__private::Pin<&#lifetime mut (#ty)>));
                proj_ref_fields
                    .push(quote!(#vis #ident: ::pin_project::__private::Pin<&#lifetime (#ty)>));
                proj_own_fields
                    .push(quote!(#vis #ident: ::pin_project::__private::PhantomData<#ty>));
                proj_body
                    .push(quote!(#ident: ::pin_project::__private::Pin::new_unchecked(#ident)));
                proj_move.push(quote!(#ident: ::pin_project::__private::PhantomData));
            } else {
                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote!(#vis #ident: &#lifetime mut (#ty)));
                proj_ref_fields.push(quote!(#vis #ident: &#lifetime (#ty)));
                proj_own_fields.push(quote!(#vis #ident: #ty));
                proj_body.push(quote!(#ident));
                proj_move.push(quote!(#ident: ::pin_project::__private::ptr::read(#ident)));
            }
            proj_pat.push(ident);
        }

        let proj_pat = quote!({ #(#proj_pat),* });
        let proj_body = quote!({ #(#proj_body),* });
        let proj_fields = quote!({ #(#proj_fields),* });
        let proj_ref_fields = quote!({ #(#proj_ref_fields),* });
        let proj_own_fields = quote!({ #(#proj_own_fields),* });
        let proj_move = quote!({ #(#proj_move),* });

        Ok(ProjectedFields {
            proj_pat,
            proj_body,
            proj_fields,
            proj_ref_fields,
            proj_own_fields,
            proj_move,
            proj_drop,
        })
    }

    fn visit_unnamed(
        &mut self,
        FieldsUnnamed { unnamed: fields, .. }: &FieldsUnnamed,
    ) -> Result<ProjectedFields> {
        let mut proj_pat = Vec::with_capacity(fields.len());
        let mut proj_body = Vec::with_capacity(fields.len());
        let mut proj_fields = Vec::with_capacity(fields.len());
        let mut proj_ref_fields = Vec::with_capacity(fields.len());
        let mut proj_own_fields = Vec::with_capacity(fields.len());
        let mut proj_move = Vec::with_capacity(fields.len());
        let mut proj_drop = Vec::with_capacity(fields.len());

        for (i, Field { attrs, vis, ty, .. }) in fields.iter().enumerate() {
            let id = format_ident!("_{}", i);
            if attrs.position_exact(PIN)?.is_some() {
                self.pinned_fields.push(ty.clone());
                proj_drop.push(id.clone());

                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote!(#vis ::pin_project::__private::Pin<&#lifetime mut (#ty)>));
                proj_ref_fields.push(quote!(#vis ::pin_project::__private::Pin<&#lifetime (#ty)>));
                proj_own_fields.push(quote!(#vis ::pin_project::__private::PhantomData<#ty>));
                proj_body.push(quote!(::pin_project::__private::Pin::new_unchecked(#id)));
                proj_move.push(quote!(::pin_project::__private::PhantomData));
            } else {
                let lifetime = &self.proj.lifetime;
                proj_fields.push(quote!(#vis &#lifetime mut (#ty)));
                proj_ref_fields.push(quote!(#vis &#lifetime (#ty)));
                proj_own_fields.push(quote!(#vis #ty));
                proj_body.push(quote!(#id));
                proj_move.push(quote!(::pin_project::__private::ptr::read(#id)));
            }
            proj_pat.push(id);
        }

        let proj_pat = quote!((#(#proj_pat),*));
        let proj_body = quote!((#(#proj_body),*));
        let proj_fields = quote!((#(#proj_fields),*));
        let proj_ref_fields = quote!((#(#proj_ref_fields),*));
        let proj_own_fields = quote!((#(#proj_own_fields),*));
        let proj_move = quote!((#(#proj_move),*));

        Ok(ProjectedFields {
            proj_pat,
            proj_body,
            proj_fields,
            proj_ref_fields,
            proj_own_fields,
            proj_move,
            proj_drop,
        })
    }

    /// Creates `Unpin` implementation for original type.
    fn make_unpin_impl(&self) -> TokenStream {
        match self.unpin_impl {
            UnpinImpl::Unsafe(span) => {
                let mut proj_generics = self.proj.generics.clone();
                let orig_ident = self.orig.ident;
                let lifetime = &self.proj.lifetime;

                // Make the error message highlight `UnsafeUnpin` argument.
                proj_generics.make_where_clause().predicates.push(parse_quote_spanned! { span =>
                    ::pin_project::__private::Wrapper<#lifetime, Self>: ::pin_project::UnsafeUnpin
                });

                let (impl_generics, _, where_clause) = proj_generics.split_for_impl();
                let ty_generics = self.orig.generics.split_for_impl().1;

                quote! {
                    impl #impl_generics ::pin_project::__private::Unpin for #orig_ident #ty_generics
                    #where_clause
                    {
                    }
                }
            }
            UnpinImpl::Negative(span) => {
                let mut proj_generics = self.proj.generics.clone();
                let orig_ident = self.orig.ident;
                let lifetime = &self.proj.lifetime;

                proj_generics.make_where_clause().predicates.push(parse_quote! {
                    ::pin_project::__private::Wrapper<
                        #lifetime, ::pin_project::__private::PhantomPinned
                    >: ::pin_project::__private::Unpin
                });

                let (proj_impl_generics, _, proj_where_clause) = proj_generics.split_for_impl();
                let (impl_generics, ty_generics, orig_where_clause) =
                    self.orig.generics.split_for_impl();

                // For interoperability with `forbid(unsafe_code)`, `unsafe` token should be
                // call-site span.
                let unsafety = token::Unsafe::default();
                quote_spanned! { span =>
                    impl #proj_impl_generics ::pin_project::__private::Unpin
                        for #orig_ident #ty_generics
                    #proj_where_clause
                    {
                    }

                    // A dummy impl of `UnsafeUnpin`, to ensure that the user cannot implement it.
                    //
                    // To ensure that users don't accidentally write a non-functional `UnsafeUnpin`
                    // impls, we emit one ourselves. If the user ends up writing an `UnsafeUnpin`
                    // impl, they'll get a "conflicting implementations of trait" error when
                    // coherence checks are run.
                    #unsafety impl #impl_generics ::pin_project::UnsafeUnpin
                        for #orig_ident #ty_generics
                    #orig_where_clause
                    {
                    }
                }
            }
            UnpinImpl::Default => {
                let mut full_where_clause =
                    self.orig.generics.where_clause.as_ref().cloned().unwrap();

                // Generate a field in our new struct for every
                // pinned field in the original type.
                let fields = self.pinned_fields.iter().enumerate().map(|(i, ty)| {
                    let field_ident = format_ident!("__field{}", i);
                    quote!(#field_ident: #ty)
                });

                // We could try to determine the subset of type parameters
                // and lifetimes that are actually used by the pinned fields
                // (as opposed to those only used by unpinned fields).
                // However, this would be tricky and error-prone, since
                // it's possible for users to create types that would alias
                // with generic parameters (e.g. 'struct T').
                //
                // Instead, we generate a use of every single type parameter
                // and lifetime used in the original struct. For type parameters,
                // we generate code like this:
                //
                // ```rust
                // struct AlwaysUnpin<T: ?Sized>(PhantomData<T>) {}
                // impl<T: ?Sized> Unpin for AlwaysUnpin<T> {}
                //
                // ...
                // _field: AlwaysUnpin<(A, B, C)>
                // ```
                //
                // This ensures that any unused type parameters
                // don't end up with `Unpin` bounds.
                let lifetime_fields = self.orig.generics.lifetimes().enumerate().map(
                    |(i, LifetimeDef { lifetime, .. })| {
                        let field_ident = format_ident!("__lifetime{}", i);
                        quote!(#field_ident: &#lifetime ())
                    },
                );

                let orig_ident = self.orig.ident;
                let struct_ident = format_ident!("__{}", orig_ident);
                let vis = self.orig.vis;
                let lifetime = &self.proj.lifetime;
                let type_params = self.orig.generics.type_params().map(|t| &t.ident);
                let proj_generics = &self.proj.generics;
                let (proj_impl_generics, proj_ty_generics, _) = proj_generics.split_for_impl();
                let (impl_generics, ty_generics, where_clause) =
                    self.orig.generics.split_for_impl();

                full_where_clause.predicates.push(syn::parse_quote! {
                    #struct_ident #proj_ty_generics: ::pin_project::__private::Unpin
                });

                quote! {
                    // This needs to have the same visibility as the original type,
                    // due to the limitations of the 'public in private' error.
                    //
                    // Our goal is to implement the public trait `Unpin` for
                    // a potentially public user type. Because of this, rust
                    // requires that any types mentioned in the where clause of
                    // our `Unpin` impl also be public. This means that our generated
                    // `__UnpinStruct` type must also be public.
                    // However, we ensure that the user can never actually reference
                    // this 'public' type by creating this type in the inside of `const`.
                    #vis struct #struct_ident #proj_generics #where_clause {
                        __pin_project_use_generics: ::pin_project::__private::AlwaysUnpin<
                            #lifetime, (#(#type_params),*)
                        >,

                        #(#fields,)*
                        #(#lifetime_fields,)*
                    }

                    impl #proj_impl_generics ::pin_project::__private::Unpin
                        for #orig_ident #ty_generics
                    #full_where_clause
                    {
                    }

                    // A dummy impl of `UnsafeUnpin`, to ensure that the user cannot implement it.
                    //
                    // To ensure that users don't accidentally write a non-functional `UnsafeUnpin`
                    // impls, we emit one ourselves. If the user ends up writing an `UnsafeUnpin`
                    // impl, they'll get a "conflicting implementations of trait" error when
                    // coherence checks are run.
                    unsafe impl #impl_generics ::pin_project::UnsafeUnpin
                        for #orig_ident #ty_generics
                    #where_clause
                    {
                    }
                }
            }
        }
    }

    /// Creates `Drop` implementation for original type.
    fn make_drop_impl(&self) -> TokenStream {
        let ident = self.orig.ident;
        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();

        if let Some(span) = self.pinned_drop {
            // For interoperability with `forbid(unsafe_code)`, `unsafe` token should be
            // call-site span.
            let unsafety = token::Unsafe::default();
            quote_spanned! { span =>
                impl #impl_generics ::pin_project::__private::Drop for #ident #ty_generics
                #where_clause
                {
                    fn drop(&mut self) {
                        // Safety - we're in 'drop', so we know that 'self' will
                        // never move again.
                        let pinned_self = #unsafety {
                            ::pin_project::__private::Pin::new_unchecked(self)
                        };
                        // We call `pinned_drop` only once. Since `PinnedDrop::drop`
                        // is an unsafe method and a private API, it is never called again in safe
                        // code *unless the user uses a maliciously crafted macro*.
                        #unsafety {
                            ::pin_project::__private::PinnedDrop::drop(pinned_self);
                        }
                    }
                }
            }
        } else {
            // If the user does not provide a `PinnedDrop` impl,
            // we need to ensure that they don't provide a `Drop` impl of their
            // own.
            // Based on https://github.com/upsuper/assert-impl/blob/f503255b292ab0ba8d085b657f4065403cfa46eb/src/lib.rs#L80-L87
            //
            // We create a new identifier for each struct, so that the traits
            // for different types do not conflict with each other.
            //
            // Another approach would be to provide an empty Drop impl,
            // which would conflict with a user-provided Drop impl.
            // However, this would trigger the compiler's special handling
            // of Drop types (e.g. fields cannot be moved out of a Drop type).
            // This approach prevents the creation of needless Drop impls,
            // giving users more flexibility.
            let trait_ident = format_ident!("{}MustNotImplDrop", ident);

            quote! {
                // There are two possible cases:
                // 1. The user type does not implement Drop. In this case,
                // the first blanked impl will not apply to it. This code
                // will compile, as there is only one impl of MustNotImplDrop for the user type
                // 2. The user type does impl Drop. This will make the blanket impl applicable,
                // which will then conflict with the explicit MustNotImplDrop impl below.
                // This will result in a compilation error, which is exactly what we want.
                trait #trait_ident {}
                #[allow(clippy::drop_bounds)]
                impl<T: ::pin_project::__private::Drop> #trait_ident for T {}
                impl #impl_generics #trait_ident for #ident #ty_generics #where_clause {}

                // A dummy impl of `PinnedDrop`, to ensure that the user cannot implement it.
                // Since the user did not pass `PinnedDrop` to `#[pin_project]`, any `PinnedDrop`
                // impl will not actually be called. Unfortunately, we can't detect this situation
                // directly from either the `#[pin_project]` or `#[pinned_drop]` attributes, since
                // we don't know what other attirbutes/impl may exist.
                //
                // To ensure that users don't accidentally write a non-functional `PinnedDrop`
                // impls, we emit one ourselves. If the user ends up writing a `PinnedDrop` impl,
                // they'll get a "conflicting implementations of trait" error when coherence
                // checks are run.
                impl #impl_generics ::pin_project::__private::PinnedDrop for #ident #ty_generics
                #where_clause
                {
                    unsafe fn drop(self: ::pin_project::__private::Pin<&mut Self>) {}
                }
            }
        }
    }

    /// Creates an implementation of the projection method.
    fn make_proj_impl(
        &self,
        proj_body: &TokenStream,
        proj_ref_body: &TokenStream,
        proj_own_body: &TokenStream,
    ) -> TokenStream {
        let vis = &self.proj.vis;
        let lifetime = &self.proj.lifetime;
        let orig_ident = self.orig.ident;
        let proj_ident = &self.proj.mut_ident;
        let proj_ref_ident = &self.proj.ref_ident;
        let proj_own_ident = &self.proj.own_ident;

        let orig_ty_generics = self.orig.generics.split_for_impl().1;
        let proj_ty_generics = self.proj.generics.split_for_impl().1;
        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();

        let replace_impl = self.replace.map(|span| {
            // For interoperability with `forbid(unsafe_code)`, `unsafe` token should be
            // call-site span.
            let unsafety = token::Unsafe::default();
            quote_spanned! { span =>
                #vis fn project_replace(
                    self: ::pin_project::__private::Pin<&mut Self>,
                    __replacement: Self,
                ) -> #proj_own_ident #orig_ty_generics {
                    #unsafety {
                        #proj_own_body
                    }
                }
            }
        });

        quote! {
            impl #impl_generics #orig_ident #ty_generics #where_clause {
                #vis fn project<#lifetime>(
                    self: ::pin_project::__private::Pin<&#lifetime mut Self>,
                ) -> #proj_ident #proj_ty_generics {
                    unsafe {
                        #proj_body
                    }
                }
                #vis fn project_ref<#lifetime>(
                    self: ::pin_project::__private::Pin<&#lifetime Self>,
                ) -> #proj_ref_ident #proj_ty_generics {
                    unsafe {
                        #proj_ref_body
                    }
                }
                #replace_impl
            }
        }
    }

    fn ensure_not_packed(&self, fields: &Fields) -> Result<TokenStream> {
        for meta in self.orig.attrs.iter().filter_map(|attr| attr.parse_meta().ok()) {
            if let Meta::List(list) = meta {
                if list.path.is_ident("repr") {
                    for repr in list.nested.iter() {
                        match repr {
                            NestedMeta::Meta(Meta::Path(path))
                            | NestedMeta::Meta(Meta::List(MetaList { path, .. }))
                                if path.is_ident("packed") =>
                            {
                                return Err(error!(
                                    repr,
                                    "#[pin_project] attribute may not be used on #[repr(packed)] types"
                                ));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // As proc-macro-derive can't rewrite the structure definition,
        // it's probably no longer necessary, but it keeps it for now.

        // Workaround for https://github.com/taiki-e/pin-project/issues/32
        // Through the tricky use of proc macros, it's possible to bypass
        // the above check for the `repr` attribute.
        // To ensure that it's impossible to use pin projections on a `#[repr(packed)]`
        // struct, we generate code like this:
        //
        // ```rust
        // #[deny(safe_packed_borrows)]
        // fn assert_not_repr_packed(val: &MyStruct) {
        //     let _field1 = &val.field1;
        //     let _field2 = &val.field2;
        //     ...
        //     let _fieldn = &val.fieldn;
        // }
        // ```
        //
        // Taking a reference to a packed field is unsafe, and applying
        // `#[deny(safe_packed_borrows)]` makes sure that doing this without
        // an `unsafe` block (which we deliberately do not generate)
        // is a hard error.
        //
        // If the struct ends up having `#[repr(packed)]` applied somehow,
        // this will generate an (unfriendly) error message. Under all reasonable
        // circumstances, we'll detect the `#[repr(packed)]` attribute, and generate
        // a much nicer error above.
        //
        // There is one exception: If the type of a struct field has an alignment of 1
        // (e.g. u8), it is always safe to take a reference to it, even if the struct
        // is `#[repr(packed)]`. If the struct is composed entirely of types of
        // alignment 1, our generated method will not trigger an error if the
        // struct is `#[repr(packed)]`.
        //
        // Fortunately, this should have no observable consequence - `#[repr(packed)]`
        // is essentially a no-op on such a type. Nevertheless, we include a test
        // to ensure that the compiler doesn't ever try to copy the fields on
        // such a struct when trying to drop it - which is reason we prevent
        // `#[repr(packed)]` in the first place.
        //
        // See also https://github.com/taiki-e/pin-project/pull/34.
        let mut field_refs = vec![];
        match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                for Field { ident, .. } in named {
                    field_refs.push(quote!(&val.#ident;));
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                for (index, _) in unnamed.iter().enumerate() {
                    let index = Index::from(index);
                    field_refs.push(quote!(&val.#index;));
                }
            }
            Fields::Unit => {}
        }

        let (impl_generics, ty_generics, where_clause) = self.orig.generics.split_for_impl();
        let ident = self.orig.ident;
        Ok(quote! {
            #[deny(safe_packed_borrows)]
            fn __assert_not_repr_packed #impl_generics (val: &#ident #ty_generics) #where_clause {
                #(#field_refs)*
            }
        })
    }
}
