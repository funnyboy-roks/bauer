use std::{
    collections::{HashMap, hash_map::Entry},
    ops::Range,
};

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{DeriveInput, Ident, Token, Type, TypePath, parse_quote, spanned::Spanned};

use crate::{
    BuilderAttr, BuilderField, Repeat,
    field::FieldIdents,
    type_state::generics::{CustomImplGenerics, CustomTypeGenerics},
    util::ReplaceTrait,
};

mod generics;

macro_rules! bail {
    ($span: expr => $message: literal $(, $args: expr)*$(,)?) => {
        return Err(syn::Error::new(
            $span,
            format!($message, $($args),*),
        ))
    }
}

fn expanded_tuple(base: TokenStream, depth: usize) -> TokenStream {
    let mut out = base;
    for _ in 0..depth {
        out = quote! { (#out, ()) };
    }
    out
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
enum Len<'a> {
    Int(usize),
    Range {
        start: usize,
        end: Option<usize>,
        inclusive: bool,
        pat: &'a syn::Pat,
    },
}

impl PartialOrd for Len<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Len<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Len::Int(a), Len::Int(b)) => a.cmp(b),
            (Len::Int(a), Len::Range { start, .. }) => a.cmp(start),
            (Len::Range { start, .. }, Len::Int(b)) => start.cmp(b),
            (
                Len::Range {
                    start: a_start,
                    end: a_end,
                    ..
                },
                Len::Range {
                    start: b_start,
                    end: b_end,
                    ..
                },
            ) => a_start.cmp(b_start).then(
                a_end
                    .unwrap_or(usize::MAX)
                    .cmp(&b_end.unwrap_or(usize::MAX)),
            ),
        }
    }
}

impl Len<'_> {
    fn range(self) -> Range<usize> {
        match self {
            Len::Int(n) => n..n + 1,
            Len::Range {
                start, end: None, ..
            } => start..usize::MAX,
            Len::Range {
                start,
                end: Some(end),
                inclusive,
                ..
            } => start..end + usize::from(inclusive),
        }
    }

    fn into_trait(self, out: &mut TokenStream) -> syn::Result<Ident> {
        match self {
            Len::Int(len) => {
                let ident = format_ident!("Eq_{}", len);
                let expanded = expanded_tuple(quote! { () }, len);
                out.extend(quote! {
                    #[allow(non_camel_case_types)]
                    trait #ident {}
                    impl #ident for #expanded {}
                });
                Ok(ident)
            }
            Len::Range {
                start,
                end: Some(end),
                inclusive,
                #[allow(unused)] // used by the unlimited_range feature section below
                pat,
            } => {
                if start >= end {
                    bail!(start.span() => "start must be less than end");
                }

                let range = self.range();

                #[cfg(not(feature = "unlimited_range"))]
                if range.len() > 64 {
                    bail!(
                        pat.span() =>
                        "Range length is limited to 64 by default as big ranges slow compile-time.  This setting may be overridden with the `unlimited_range` feature.  Alternatively, half-open ranges like `5..` and integer constants are faster"
                    );
                }

                let ident = format_ident!(
                    "Range_{}_{}{}",
                    start,
                    end,
                    if inclusive { "_Inclusive" } else { "" },
                );
                out.extend(quote! {
                    #[allow(non_camel_case_types)]
                    trait #ident {}
                });

                for i in range {
                    let expanded = expanded_tuple(quote! { () }, i);
                    out.extend(quote! {
                        impl #ident for #expanded {}
                    });
                }

                Ok(ident)
            }
            Len::Range {
                start, end: None, ..
            } => {
                let ident = format_ident!("Gte_{}", start);
                let expanded = expanded_tuple(quote! { T }, start);
                out.extend(quote! {
                    #[allow(non_camel_case_types)]
                    trait #ident {}
                    impl<T> #ident for #expanded {}
                });
                Ok(ident)
            }
        }
    }
}

impl<'a> TryFrom<&'a syn::Pat> for Len<'a> {
    type Error = syn::Error;

    fn try_from(pat: &'a syn::Pat) -> Result<Self, Self::Error> {
        let v = match pat {
            syn::Pat::Lit(syn::ExprLit {
                lit: syn::Lit::Int(int),
                ..
            }) => {
                let len = int.base10_parse()?;
                Len::Int(len)
            }
            syn::Pat::Range(syn::ExprRange {
                start: Some(start),
                end: Some(end),
                limits,
                ..
            }) => {
                let start: usize = match &**start {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(start),
                        ..
                    }) => start.base10_parse()?,
                    _ => {
                        bail!(start.span() => "start must be an integer literal");
                    }
                };

                let end: usize = match &**end {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(end),
                        ..
                    }) => end.base10_parse()?,
                    _ => {
                        bail!(end.span() => "end must be an integer literal");
                    }
                };

                match limits {
                    syn::RangeLimits::HalfOpen(_) => Len::Range {
                        start,
                        end: Some(end),
                        inclusive: false,
                        pat,
                    },
                    syn::RangeLimits::Closed(_) => Len::Range {
                        start,
                        end: Some(end),
                        inclusive: true,
                        pat,
                    },
                }
            }
            syn::Pat::Range(syn::ExprRange {
                start: None,
                end: Some(end),
                limits,
                ..
            }) => {
                let end: usize = match &**end {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(end),
                        ..
                    }) => end.base10_parse()?,
                    _ => {
                        bail!(end.span() => "end must be an integer literal");
                    }
                };

                match limits {
                    syn::RangeLimits::HalfOpen(_) => Len::Range {
                        start: 0,
                        end: Some(end),
                        inclusive: false,
                        pat,
                    },
                    syn::RangeLimits::Closed(_) => Len::Range {
                        start: 0,
                        end: Some(end),
                        inclusive: true,
                        pat,
                    },
                }
            }
            syn::Pat::Range(syn::ExprRange {
                start: Some(start),
                end: None,
                ..
            }) => {
                let start: usize = match &**start {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Int(int),
                        ..
                    }) => int.base10_parse()?,
                    _ => {
                        bail!(start.span() => "start must be an integer literal");
                    }
                };

                Len::Range {
                    start,
                    end: None,
                    inclusive: false,
                    pat,
                }
            }
            _ => {
                bail!(pat.span() => "repeat_n on type-state builders can only use integer literals and ranges");
            }
        };
        Ok(v)
    }
}

fn build_fn(
    builder: &Ident,
    builder_attr: &BuilderAttr,
    fields: &[BuilderField],
    generic_fields: &[&BuilderField],
    len_structs: &HashMap<usize, Ident>,
    input: &DeriveInput,
    inner: &Ident,
) -> TokenStream {
    let ident = &input.ident;
    let builder_vis = &builder_attr.vis;

    let build_fields = fields.iter().map(|field| {
        let name = &field.ident;
        let field_i = field.tuple_index();

        if let Some(Repeat { inner_ty, .. }) = &field.attr.repeat {
            quote_spanned! {
                inner_ty.span() =>
                // using associated function syntax as that gives better error messages
                // (i.e., not "call chain may not have expected associated type"
                #name: ::std::iter::FromIterator::from_iter(inner.#field_i.into_iter())
            }
        } else if field.wrapped_option {
            quote! {
                #name: inner.#field_i
            }
        } else if let Some(default) = &field.attr.default {
            if let Some(default) = default {
                if field.attr.into {
                    quote! {
                        #name: inner.#field_i.unwrap_or_else(|| #default.into())
                    }
                } else {
                    quote! {
                        #name: inner.#field_i.unwrap_or_else(|| #default)
                    }
                }
            } else {
                quote_spanned! {
                    field.ty.span() =>
                    #name: inner.#field_i.unwrap_or_default()
                }
            }
        } else {
            quote! {
                #name: inner.#field_i.unwrap()
            }
        }
    });

    let build_impl_generics = generic_fields.iter().enumerate().filter_map(|(i, f)| {
        if f.optional() || len_structs.contains_key(&i) {
            Some(&f.idents.pascal)
        } else {
            None
        }
    });

    let build_generics = generic_fields.iter().enumerate().map(|(i, f)| {
        let FieldIdents {
            count, pascal, set, ..
        } = &f.idents;
        if len_structs.contains_key(&i) {
            let ty: Type = parse_quote! { #count<#pascal> };
            ty
        } else if f.optional() {
            Type::from(TypePath {
                qself: None,
                path: pascal.clone().into(),
            })
        } else {
            Type::from(TypePath {
                qself: None,
                path: set.clone().into(),
            })
        }
    });

    let impl_generics = CustomImplGenerics::new(&input.generics, build_impl_generics);
    let ty_generics = CustomTypeGenerics::new(&input.generics, build_generics);

    let (_, _, where_clause) = input.generics.split_for_impl();

    let mut builder_where = where_clause.to_token_stream();
    if let Some(where_clause) = where_clause {
        if !where_clause.predicates.trailing_punct() {
            <Token![,]>::default().to_tokens(&mut builder_where);
        }
    } else {
        <Token![where]>::default().to_tokens(&mut builder_where);
    }

    for (&i, len) in len_structs {
        let generic = &generic_fields[i].idents.pascal;
        builder_where.extend(quote! {
            #generic: #len,
        });
    }

    let (_, default_ty_generics, _) = input.generics.split_for_impl();

    quote! {
        impl #impl_generics #builder #ty_generics #builder_where {
            #builder_vis fn build(self) -> #ident #default_ty_generics  {
                #[allow(deprecated)] // #inner is set to deprecated
                {
                    let inner = self.#inner;
                    #ident {
                        #(#build_fields),*
                    }
                }
            }
        }

        impl #impl_generics ::core::convert::From<#builder #ty_generics> for #ident #default_ty_generics #builder_where {
            fn from(builder: #builder #ty_generics) -> Self  {
                builder.build()
            }
        }
    }
}

pub fn type_state_builder(
    builder_attr: &BuilderAttr,
    input: &DeriveInput,
    fields: &[BuilderField],
) -> TokenStream {
    let ident = &input.ident;
    let builder = format_ident!("{}Builder", ident);
    let builder_vis = &builder_attr.vis;

    let generic_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.attr.repeat.as_ref().is_none_or(|r| r.len.is_some()))
        .collect();

    let mut out = TokenStream::new();

    out.extend(generic_fields.iter().map(|&f| {
        let FieldIdents {
            count, set, unset, ..
        } = &f.idents;
        if f.attr.repeat.as_ref().is_some_and(|r| r.len.is_some()) {
            quote! {
                #[doc(hidden)]
                #[non_exhaustive]
                struct #count<T>(T); // never constructed, so doesn't really need to be PhantomData
            }
        } else {
            quote! {
                #[doc(hidden)]
                #[non_exhaustive]
                struct #set;
                #[doc(hidden)]
                #[non_exhaustive]
                struct #unset;
            }
        }
    }));

    let mut len_structs = HashMap::new();

    let mut len_traits = HashMap::<Len, Ident>::new();

    for (i, &f) in generic_fields.iter().enumerate() {
        let Some(repeat) = &f.attr.repeat else {
            continue;
        };
        let Some((len_pat, _)) = &repeat.len else {
            continue;
        };

        let len = match Len::try_from(len_pat) {
            Ok(v) => v,
            Err(e) => return e.to_compile_error(),
        };

        let ident = match len_traits.entry(len) {
            Entry::Occupied(entry) => entry.get().clone(),
            Entry::Vacant(entry) => {
                let ident = match len.into_trait(&mut out) {
                    Ok(i) => i,
                    Err(e) => return e.to_compile_error(),
                };
                entry.insert(ident.clone());
                ident
            }
        };
        len_structs.insert(i, ident);
    }

    let (default_impl_generics, default_ty_generics, where_clause) =
        input.generics.split_for_impl();

    let field_decls = fields.iter().map(|f| {
        if let Some(Repeat { inner_ty, .. }) = &f.attr.repeat {
            quote! { ::std::vec::Vec<#inner_ty> }
        } else {
            let ty = &f.ty;
            quote! { ::core::option::Option<#ty> }
        }
    });

    let inner = format_ident!("__unsafe_builder_content");
    let state = format_ident!("__unsafe_builder_state");

    let phantom_generics = generic_fields.iter().map(|f| &f.idents.pascal);
    let phantom = quote! {
        #state: ::core::marker::PhantomData<(#(#phantom_generics,)*)>
    };

    let new_generics = generic_fields.iter().map(|f| {
        let FieldIdents { count, unset, .. } = &f.idents;
        if f.attr.repeat.as_ref().is_some_and(|f| f.len.is_some()) {
            quote! { #count<()> }
        } else {
            unset.to_token_stream()
        }
    });

    let struct_generics = CustomImplGenerics::new(
        &input.generics,
        generic_fields.iter().map(|f| &f.idents.pascal),
    );
    let new_generics = CustomTypeGenerics::new(&input.generics, new_generics);

    let init = fields
        .iter()
        .map(|_| quote! { ::core::default::Default::default() });

    out.extend(quote! {
        #[allow(clippy::type_complexity)]
        #builder_vis struct #builder #struct_generics {
            #[deprecated = "This field is for internal use only; You almost certainly don't need to touch this. If you encounter a bug or missing feature, file an issue on the repo."]
            #[doc(hidden)]
            #inner: (#(#field_decls,)*),
            #[deprecated = "This field is for internal use only; You almost certainly don't need to touch this. If you encounter a bug or missing feature, file an issue on the repo."]
            #[doc(hidden)]
            #phantom
        }

        impl #default_impl_generics #ident #default_ty_generics #where_clause {
            #builder_vis fn builder() -> #builder #new_generics {
                #builder::new()
            }
        }

        impl #default_impl_generics #builder #new_generics #where_clause {
            #builder_vis fn new() -> Self {
                Self {
                    #inner: (#(#init,)*),
                    #state: ::core::marker::PhantomData,
                }
            }
        }
    });

    // add `fn build()`
    out.extend(build_fn(
        &builder,
        builder_attr,
        fields,
        &generic_fields,
        &len_structs,
        input,
        &inner,
    ));

    let mut i = 0;
    for f in fields {
        let (args, value) = f.attr.to_args_and_value(f.arg_ty(), &f.ident);
        let fn_ident = f.function_ident(builder_attr);

        let doc = &f.doc;

        let field_i = f.tuple_index();
        let fun = match &f.attr.repeat {
            Some(Repeat { len: None, .. }) => {
                let impl_generics = CustomImplGenerics::new(
                    &input.generics,
                    generic_fields.iter().map(|f| &f.idents.pascal),
                );
                let ty_generics = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields.iter().map(|f| &f.idents.pascal),
                );
                quote_spanned! {
                    fn_ident.span() =>
                    impl #impl_generics #builder #ty_generics #where_clause {
                        #(#doc)*
                        #[allow(clippy::type_complexity)]
                        pub fn #fn_ident(self, #args) -> #builder #ty_generics {
                            let mut this = self; // rather than have `mut self` in the signature
                            #[allow(deprecated)] // #inner is set to deprecated
                            {
                                this.#inner.#field_i.push(#value);
                                #builder {
                                    #inner: this.#inner,
                                    #state: ::core::marker::PhantomData,
                                }
                            }
                        }
                    }
                }
            }
            Some(Repeat { len: Some(_), .. }) => {
                let FieldIdents { count, pascal, .. } = &generic_fields[i].idents;

                fn ident_to_type(ident: Ident) -> Type {
                    TypePath {
                        qself: None,
                        path: ident.into(),
                    }
                    .into()
                }

                let impl_generics = CustomImplGenerics::new(
                    &input.generics,
                    generic_fields.iter().map(|f| &f.idents.pascal),
                );
                let ty_generics = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| ident_to_type(f.idents.pascal.clone()))
                        .replace(i, parse_quote! { #count<#pascal> }),
                );

                let ret_ty_generics = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| ident_to_type(f.idents.pascal.clone()))
                        .replace(i, parse_quote! { #count<(#pascal, ())> }),
                );

                quote_spanned! {
                    fn_ident.span() =>
                    impl #impl_generics #builder #ty_generics #where_clause {
                        #(#doc)*
                        #[allow(clippy::type_complexity)]
                        pub fn #fn_ident(self, #args) -> #builder #ret_ty_generics {
                            let mut this = self; // rather than have `mut self` in the signature
                            #[allow(deprecated)] // #inner is set to deprecated
                            {
                                this.#inner.#field_i.push(#value);
                                #builder {
                                    #inner: this.#inner,
                                    #state: ::core::marker::PhantomData,
                                }
                            }
                        }
                    }
                }
            }
            None => {
                let impl_generics_fields = CustomImplGenerics::new(
                    &input.generics,
                    generic_fields[..i]
                        .iter()
                        .chain(generic_fields.iter().skip(i + 1))
                        .map(|f| &f.idents.pascal),
                );

                let struct_generics_fields = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| &f.idents.pascal)
                        .replace(i, &generic_fields[i].idents.unset),
                );

                let return_struct_generics_fields = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| &f.idents.pascal)
                        .replace(i, &generic_fields[i].idents.set),
                );

                quote_spanned! {
                    fn_ident.span() =>
                    impl #impl_generics_fields #builder #struct_generics_fields #where_clause {
                        #(#doc)*
                        #[allow(clippy::type_complexity)]
                        pub fn #fn_ident(self, #args) -> #builder #return_struct_generics_fields {
                            let mut this = self; // rather than have `mut self` in the signature
                            #[allow(deprecated)] // #inner is set to deprecated
                            {
                                this.#inner.#field_i = Some(#value);
                                #builder {
                                    #inner: this.#inner,
                                    #state: ::core::marker::PhantomData,
                                }
                            }
                        }
                    }
                }
            }
        };

        out.extend(fun);
        if f.attr.repeat.as_ref().is_none_or(|r| r.len.is_some()) {
            i += 1;
        }
    }

    out
}
