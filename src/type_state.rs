use std::{
    collections::{HashMap, HashSet, VecDeque, hash_map::Entry},
    ops::Range,
};

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{DeriveInput, Ident, Token, spanned::Spanned};

use crate::{BuilderAttr, BuilderField, Repeat};

macro_rules! bail {
    ($span: expr => $message: literal $(, $args: expr)*$(,)?) => {
        return Err(syn::Error::new(
            $span,
            format!($message, $($args),*),
        ))
    }
}

fn trim_angle_brackets(tokens: impl ToTokens) -> TokenStream {
    let mut tokens = tokens
        .to_token_stream()
        .into_iter()
        .collect::<VecDeque<_>>();
    if tokens.is_empty() {
        quote! {}
    } else {
        tokens.pop_front();
        tokens.pop_back();
        tokens.into_iter().collect()
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

    let fields_pascal: Vec<_> = generic_fields
        .iter()
        .map(|f| Ident::new(&f.ident.to_string().to_case(Case::Pascal), f.ident.span()))
        .collect();

    let (set_fields, unset_fields, count_fields): (Vec<_>, Vec<_>, Vec<_>) = fields_pascal
        .iter()
        .map(|name| {
            let set = format_ident!("{}{}Set", ident, name);
            let unset = format_ident!("{}{}Unset", ident, name);
            let count = format_ident!("{}{}Count", ident, name);
            (set, unset, count)
        })
        .collect();

    out.extend(
        generic_fields
            .iter()
            .zip(set_fields.iter())
            .zip(unset_fields.iter())
            .zip(count_fields.iter())
            .map(|(((&f, set), unset), count)| {
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
            }),
    );

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

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let impl_generics = trim_angle_brackets(impl_generics);
    let ty_generics = trim_angle_brackets(ty_generics);

    let field_decls: TokenStream = fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if let Some(Repeat { inner_ty, .. }) = &f.attr.repeat {
                quote! {
                    #ident: ::std::vec::Vec<#inner_ty>,
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #ident: ::core::option::Option<#ty>,
                }
            }
        })
        .collect();

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    let mut state = "_state".to_string();
    let names_set = field_names
        .iter()
        .map(|i| i.to_string())
        .collect::<HashSet<_>>();
    while names_set.contains(&*state) {
        state = format!("_{}", state);
    }
    let state = Ident::new(&state, Span::call_site());

    let phantom = quote! {
        #state: ::core::marker::PhantomData<(#(#fields_pascal,)*)>
    };

    let build_fields = fields.iter().map(|field| {
        let name = &field.ident;

        if let Some(Repeat { inner_ty, .. }) = &field.attr.repeat {
            quote_spanned! {
                inner_ty.span() =>
                // using associated function syntax as that gives better error messages
                // (i.e., not "call chain may not have expected associated type"
                #name: ::std::iter::FromIterator::from_iter(self.#name.into_iter())
            }
        } else if field.wrapped_option {
            quote! {
                #name: self.#name
            }
        } else if let Some(default) = &field.attr.default {
            if let Some(default) = default {
                if field.attr.into {
                    quote! {
                        #name: self.#name.unwrap_or_else(|| #default.into())
                    }
                } else {
                    quote! {
                        #name: self.#name.unwrap_or_else(|| #default)
                    }
                }
            } else {
                quote_spanned! {
                    field.ty.span() =>
                    #name: self.#name.unwrap_or_default()
                }
            }
        } else {
            quote! {
                #name: self.#name.unwrap()
            }
        }
    });

    let build_impl_generics = fields
        .iter()
        .enumerate()
        .zip(fields_pascal.iter())
        .filter_map(|((i, f), name)| {
            if f.optional() || len_structs.contains_key(&i) {
                Some(name)
            } else {
                None
            }
        });

    let build_generics = fields
        .iter()
        .enumerate()
        .zip(fields_pascal.iter())
        .zip(set_fields.iter())
        .zip(count_fields.iter())
        .map(|((((i, f), pascal), set), count)| {
            if len_structs.contains_key(&i) {
                quote! { #count<#pascal> }
            } else if f.optional() {
                pascal.to_token_stream()
            } else {
                set.to_token_stream()
            }
        });

    let new_generics: Vec<_> = generic_fields
        .iter()
        .zip(unset_fields.iter())
        .zip(count_fields.iter())
        .map(|((f, unset), count)| {
            if f.attr.repeat.as_ref().is_some_and(|f| f.len.is_some()) {
                quote! { #count<()> }
            } else {
                unset.to_token_stream()
            }
        })
        .collect();

    let mut builder_where = where_clause.to_token_stream();
    if let Some(where_clause) = where_clause {
        if !where_clause.predicates.trailing_punct() {
            <Token![,]>::default().to_tokens(&mut builder_where);
        }
    } else {
        <Token![where]>::default().to_tokens(&mut builder_where);
    }

    for (&i, len) in &len_structs {
        let generic = &fields_pascal[i];
        builder_where.extend(quote! {
            #generic: #len,
        });
    }

    out.extend(quote! {
        #[allow(clippy::type_complexity)]
        #builder_vis struct #builder <#(#fields_pascal,)* #ty_generics> {
            #field_decls
            #phantom
        }

        impl <#impl_generics> #ident <#ty_generics> {
            #builder_vis fn builder() -> #builder<#(#new_generics,)* #ty_generics> {
                #builder::new()
            }
        }

        impl <#impl_generics> #builder<#(#new_generics,)* #ty_generics> #where_clause {
            #builder_vis fn new() -> Self {
                Self {
                    #(#field_names: ::core::default::Default::default(),)*
                    #state: ::core::marker::PhantomData,
                }
            }
        }

        impl <#(#build_impl_generics,)* #impl_generics> #builder<#(#build_generics,)* #ty_generics> #builder_where {
            #builder_vis fn build(self) -> #ident<#ty_generics> {
                #ident {
                    #(#build_fields),*
                }
            }
        }
    });

    let mut i = 0;
    for f in fields.iter() {
        let (args, value) = f.attr.to_args_and_value(f.arg_ty(), &f.ident);
        let fn_ident = f.function_ident(builder_attr);

        let name = &f.ident;
        let doc = &f.doc;

        let fun = match &f.attr.repeat {
            Some(Repeat { len: None, .. }) => {
                quote_spanned! {
                    fn_ident.span() =>
                    impl <#(#fields_pascal,)* #impl_generics> #builder <#(#fields_pascal,)* #ty_generics> {
                        #(#doc)*
                        #[allow(clippy::type_complexity)]
                        pub fn #fn_ident(self, #args) -> #builder <#(#fields_pascal,)* #ty_generics> {
                            let mut this = self; // rather than have `mut self` in the signature
                            this.#name.push(#value);
                            #builder {
                                #(#field_names: this.#field_names,)*
                                #state: ::core::marker::PhantomData,
                            }
                        }
                    }
                }
            }
            Some(Repeat { len: Some(_), .. }) => {
                let pascal_prefix = &fields_pascal[..i];
                let pascal_suffix = &fields_pascal[(i + 1)..];
                let pascal = &fields_pascal[i];
                let count = &count_fields[i];

                quote_spanned! {
                    fn_ident.span() =>
                    impl <#(#fields_pascal,)* #impl_generics> #builder <#(#pascal_prefix,)* #count<#pascal>, #(#pascal_suffix,)* #ty_generics> {
                        #(#doc)*
                        #[allow(clippy::type_complexity)]
                        pub fn #fn_ident(self, #args) -> #builder <#(#pascal_prefix,)* #count<(#pascal, ())>, #(#pascal_suffix,)* #ty_generics> {
                            let mut this = self; // rather than have `mut self` in the signature
                            this.#name.push(#value);
                            #builder {
                                #(#field_names: this.#field_names,)*
                                #state: ::core::marker::PhantomData,
                            }
                        }
                    }
                }
            }
            None => {
                let impl_generics_fields = fields_pascal[..i]
                    .iter()
                    .chain(fields_pascal.iter().skip(i + 1));

                let struct_generics_fields = fields_pascal[..i]
                    .iter()
                    .chain(std::iter::once(&unset_fields[i]))
                    .chain(fields_pascal.iter().skip(i + 1));

                let return_struct_generics_fields = fields_pascal[..i]
                    .iter()
                    .chain(std::iter::once(&set_fields[i]))
                    .chain(fields_pascal.iter().skip(i + 1));

                quote_spanned! {
                    fn_ident.span() =>
                    impl <#(#impl_generics_fields,)* #impl_generics> #builder <#(#struct_generics_fields,)* #ty_generics> {
                        #(#doc)*
                        #[allow(clippy::type_complexity)]
                        pub fn #fn_ident(self, #args) -> #builder <#(#return_struct_generics_fields,)* #ty_generics> {
                            let mut this = self; // rather than have `mut self` in the signature
                            this.#name = Some(#value);
                            #builder {
                                #(#field_names: this.#field_names,)*
                                #state: ::core::marker::PhantomData,
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
