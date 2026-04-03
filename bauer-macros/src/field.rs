use std::ops::Range;

use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote};
use syn::{
    Expr, ExprClosure, Field, Ident, LitStr, Meta, Pat, Token, Type, parenthesized,
    parse::{Parse, ParseStream},
    parse_quote,
    spanned::Spanned,
    token::Paren,
};

use crate::{BuilderAttr, Kind};

pub(crate) fn get_single_generic<'a>(ty: &'a Type, name: Option<&str>) -> Option<&'a Type> {
    match ty {
        Type::Path(path)
            if path
                .path
                .segments
                .last()
                .is_some_and(|s| name.is_none_or(|name| s.ident == name))
                && path.path.segments.len() == 1 =>
        {
            let option = path
                .path
                .segments
                .last()
                .expect("checked in guard condition");

            let arg = match option.arguments {
                syn::PathArguments::AngleBracketed(ref args) if args.args.len() == 1 => {
                    let Some(syn::GenericArgument::Type(arg)) = args.args.first() else {
                        return None;
                    };
                    arg
                }
                _ => return None,
            };
            Some(arg)
        }
        Type::Array(arr) if name.is_none() => Some(&arr.elem),
        Type::Slice(slice) if name.is_none() => Some(&slice.elem),
        Type::Reference(r) => get_single_generic(&r.elem, name),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use syn::{Type, parse_quote};

    use super::get_single_generic;

    #[test]
    fn single_generic() {
        let inner: Type = parse_quote! { u32 };
        let full = parse_quote! { Foo<#inner> };
        let single = get_single_generic(&full, None);
        assert_eq!(Some(&inner), single);
    }

    #[test]
    fn double_generic() {
        let full: Type = parse_quote! { Foo<u32, u32> };
        let single = get_single_generic(&full, None);
        assert_eq!(None, single);
    }

    #[test]
    fn array_generic() {
        let inner: Type = parse_quote! { &str };
        let full = parse_quote! { [#inner; 6] };
        let single = get_single_generic(&full, None);
        assert_eq!(Some(&inner), single);
    }

    #[test]
    fn slice_generic() {
        let inner: Type = parse_quote! { String };
        let full = parse_quote! { [#inner] };
        let single = get_single_generic(&full, None);
        assert_eq!(Some(&inner), single);
    }

    #[test]
    fn ref_slice_generic() {
        let inner: Type = parse_quote! { u8 };
        let full = parse_quote! { &[#inner] };
        let single = get_single_generic(&full, None);
        assert_eq!(Some(&inner), single);
    }

    #[test]
    fn single_generic_name() {
        let full = parse_quote! { Foo<u32> };
        let single = get_single_generic(&full, Some("Bar"));
        assert_eq!(None, single);
    }
}

macro_rules! bail {
    ($span: expr => $message: literal $(, $args: expr)*$(,)?) => {
        return Err(syn::Error::new(
            $span,
            format!($message, $($args),*),
        ))
    }
}

#[derive(Clone, Copy)]
enum Attribute {
    Default,
    Into,
    Repeat,
    RepeatN,
    Rename,
    SkipPrefix,
    SkipSuffix,
    Tuple,
    Adapter,
}

impl Attribute {
    const ALL: [Self; 9] = [
        Self::Default,
        Self::Into,
        Self::Repeat,
        Self::RepeatN,
        Self::Rename,
        Self::SkipPrefix,
        Self::SkipSuffix,
        Self::Tuple,
        Self::Adapter,
    ];

    const fn as_str(self) -> &'static str {
        match self {
            Attribute::Default => "default",
            Attribute::Into => "into",
            Attribute::Repeat => "repeat",
            Attribute::RepeatN => "repeat_n",
            Attribute::Rename => "rename",
            Attribute::SkipPrefix => "skip_prefix",
            Attribute::SkipSuffix => "skip_suffix",
            Attribute::Tuple => "tuple",
            Attribute::Adapter => "adapter",
        }
    }
}

impl AsRef<str> for Attribute {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Attribute {
    fn parse(ident: &Ident) -> syn::Result<Self> {
        Self::ALL
            .iter()
            .copied()
            .find(|e| ident == e)
            .ok_or_else(|| {
                syn::Error::new(
                    ident.span(),
                    format!(
                        "Unknown attribute '{}'.  Valid attribute are: '{}'",
                        ident,
                        Self::ALL
                            .into_iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            })
    }
}

pub struct FieldIdents {
    pub pascal: Ident,
    pub set: Ident,
    pub count: Ident,
}

impl FieldIdents {
    fn new(struct_name: &Ident, ident: &Ident) -> Self {
        let pascal = Ident::new(&ident.to_string().to_case(Case::Pascal), ident.span());
        Self {
            set: format_ident!("{}_{}_Set", struct_name, pascal, span = pascal.span()),
            count: format_ident!("{}_{}_Count", struct_name, pascal, span = pascal.span()),
            pascal,
        }
    }
}

pub struct BuilderField {
    pub ident: Ident,
    pub ty: Type,
    pub attr: FieldAttr,
    pub missing_err: Option<Ident>,
    pub wrapped_option: bool,
    pub doc: Vec<syn::Attribute>,
    pub idents: FieldIdents,
    pub index: usize,
}

impl BuilderField {
    pub fn tuple_index(&self) -> syn::Index {
        syn::Index {
            index: self.index as _,
            span: self.ident.span(),
        }
    }

    pub fn arg_ty(&self) -> &Type {
        self.attr
            .repeat
            .as_ref()
            .map(|r| &r.inner_ty)
            .unwrap_or(&self.ty)
    }

    pub fn optional(&self) -> bool {
        self.wrapped_option || self.attr.default.is_some()
    }

    pub fn function_ident(&self, builder_attr: &BuilderAttr) -> Ident {
        let ident = self.attr.rename.as_ref().unwrap_or(&self.ident);
        let prefix = if self.attr.skip_prefix {
            ""
        } else {
            &builder_attr.prefix
        };

        let suffix = if self.attr.skip_suffix {
            ""
        } else {
            &builder_attr.suffix
        };

        format_ident!("{}{}{}", prefix, ident, suffix, span = ident.span())
    }

    pub(crate) fn function(&self, builder_attr: &BuilderAttr, inner: &Ident) -> TokenStream {
        let field_name = &self.ident;

        let ty = self.arg_ty();
        let fn_ident = self.function_ident(builder_attr);
        let (args, value) = self.attr.to_args_and_value(ty, field_name);
        let doc = &self.doc;
        let self_param = builder_attr.self_param();
        let return_type = builder_attr.return_type();
        let builder_vis = &builder_attr.vis;

        let field_i = self.tuple_index();

        let setter = if self.attr.repeat.is_some() {
            quote! { self.#inner.#field_i.push(value) }
        } else {
            quote! { self.#inner.#field_i = Some(value) }
        };

        quote! {
            #(#doc)*
                #[must_use = "The builder doesn't construct its type until `.build()` is called"]
                #builder_vis fn #fn_ident(#self_param, #args) -> #return_type {
                    let value = #value;
                    #[allow(deprecated)] // #inner is set to deprecated
                    {
                        #setter;
                    }
                    self
            }
        }
    }

    pub fn parse(
        value: &Field,
        builder_attr: &BuilderAttr,
        struct_name: &Ident,
        index: usize,
    ) -> syn::Result<Self> {
        let ident = value.ident.as_ref().expect("We only support named fields");
        let attr: FieldAttr = if let Some(attr) =
            value.attrs.iter().find(|a| a.path().is_ident("builder"))
        {
            attr.parse_args_with(|input: ParseStream| FieldAttr::parse(input, builder_attr, value))?
        } else {
            FieldAttr::default()
        };

        let (ty, wrapped_option) = if let Some(ty) = get_single_generic(&value.ty, Some("Option")) {
            (ty, true)
        } else {
            (&value.ty, false)
        };

        let doc: Vec<syn::Attribute> = value
            .attrs
            .iter()
            .filter(|a| {
                if let Meta::NameValue(meta) = &a.meta {
                    meta.path.get_ident().is_some_and(|n| n == "doc")
                } else {
                    false
                }
            })
            .cloned()
            .collect();

        Ok(BuilderField {
            ident: ident.clone(),
            ty: ty.clone(),
            missing_err: if attr.default.is_none() && attr.repeat.is_none() {
                let mut ident = format_ident!("Missing{}", ident.to_string().to_case(Case::Pascal));
                ident.set_span(value.ident.as_ref().unwrap().span());
                Some(ident)
            } else {
                None
            },
            attr,
            wrapped_option,
            doc,
            idents: FieldIdents::new(struct_name, ident),
            index,
        })
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Len {
    /// No length specified
    None,
    /// Length specified, but parsing was not necessary (not type-state builder)
    Raw {
        pattern: Pat,
        error: Ident,
    },
    Int(usize),
    Range {
        start: usize,
        end: Option<usize>,
        inclusive: bool,
        pat: syn::Pat,
    },
}

impl Len {
    pub fn is_some(&self) -> bool {
        match self {
            Len::None => false,
            Len::Raw { .. } | Len::Int(_) | Len::Range { .. } => true,
        }
    }

    pub fn is_none(&self) -> bool {
        match self {
            Len::None => true,
            Len::Raw { .. } | Len::Int(_) | Len::Range { .. } => false,
        }
    }

    fn range(&self) -> Option<Range<usize>> {
        match self {
            Len::None => None,
            Len::Raw { .. } => None,
            Len::Int(n) => Some(*n..*n + 1),
            Len::Range {
                start, end: None, ..
            } => Some(*start..usize::MAX),
            Len::Range {
                start,
                end: Some(end),
                inclusive,
                ..
            } => Some(*start..*end + usize::from(*inclusive)),
        }
    }

    fn expanded_tuple(base: Type, depth: usize) -> Type {
        let mut out = base;
        for _ in 0..depth {
            out = parse_quote! { (#out, ()) };
        }
        out
    }

    pub fn to_trait(&self, out: &mut TokenStream) -> syn::Result<Ident> {
        match self {
            Len::None => unreachable!("Len::into_trait called on None"),
            Len::Raw { .. } => unreachable!("Len::into_trait called on Raw"),
            Len::Int(len) => {
                let ident = format_ident!("Eq_{}", len);
                let expanded = Self::expanded_tuple(parse_quote! { () }, *len);
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

                let range = self.range().expect("Len::range is Some for Len::Range");

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
                    if *inclusive { "_Inclusive" } else { "" },
                );
                out.extend(quote! {
                    #[allow(non_camel_case_types)]
                    trait #ident {}
                });

                for i in range {
                    let expanded = Self::expanded_tuple(parse_quote! { () }, i);
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
                let expanded = Self::expanded_tuple(parse_quote! { T }, *start);
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

impl TryFrom<syn::Pat> for Len {
    type Error = syn::Error;

    fn try_from(pat: syn::Pat) -> Result<Self, Self::Error> {
        let v = match pat {
            syn::Pat::Lit(syn::ExprLit {
                lit: syn::Lit::Int(int),
                ..
            }) => {
                let len = int.base10_parse()?;
                Len::Int(len)
            }
            syn::Pat::Range(syn::ExprRange {
                start: Some(ref start),
                end: Some(ref end),
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
                end: Some(ref end),
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
                start: Some(ref start),
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

pub struct Repeat {
    pub inner_ty: Type,
    pub len: Len,
    pub array: bool,
}

#[derive(Debug)]
pub struct Adapter {
    pub args: Vec<(Ident, Type)>,
    pub expr: Expr,
}

impl Adapter {
    pub fn to_args_and_value(&self) -> (TokenStream, TokenStream) {
        let (names, types): (Vec<&Ident>, Vec<&Type>) =
            self.args.iter().map(|(a, b)| (a, b)).collect();

        let expr = &self.expr;

        let args = quote! {
            #(#names: #types),*
        };
        let expr = quote! { #expr };

        (args, expr)
    }
}

impl Parse for Adapter {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let closure: ExprClosure = input.parse()?;
        let mut args = Vec::with_capacity(closure.inputs.len());
        for input in closure.inputs.into_iter() {
            match input {
                syn::Pat::Type(pt) => {
                    let ident = match *pt.pat {
                        syn::Pat::Ident(pi) => pi.ident,
                        pat => bail!(pat.span() => "Expected `name: type` arguments"),
                    };
                    args.push((ident, *pt.ty));
                }
                syn::Pat::Ident(_) => {
                    bail!(input.span() => "Type missing for argument");
                }
                _ => {
                    bail!(input.span() => "Expected `name: type` arguments");
                }
            }
        }

        Ok(Self {
            args,
            expr: *closure.body,
        })
    }
}

#[derive(Default)]
pub struct FieldAttr {
    /// Some(Some(expr)) -> default is expr
    /// Some(None)       -> default is Default::default()
    /// None             -> no default
    pub default: Option<Option<Expr>>,
    pub into: bool,
    pub repeat: Option<Repeat>,
    pub rename: Option<Ident>,
    pub skip_prefix: bool,
    pub skip_suffix: bool,
    /// Some(Some(names)) -> tuple argument use `names` for names
    /// Some(None)        -> tuple argument use `field_N` for names
    /// None              -> tuple is passed as a value
    pub tuple: Option<Option<Vec<Ident>>>,
    pub adapter: Option<Adapter>,
}

impl FieldAttr {
    pub fn to_args_and_value(&self, ty: &Type, field_name: &Ident) -> (TokenStream, TokenStream) {
        if let Some(adapter) = &self.adapter {
            return adapter.to_args_and_value();
        }

        if let (Some(t), Type::Tuple(tuple)) = (&self.tuple, ty) {
            let names = t.clone().unwrap_or_else(|| {
                (0..tuple.elems.len())
                    .map(|n| format_ident!("{}_{}", field_name, n))
                    .collect()
            });

            let types = tuple.elems.iter();

            return if self.into {
                (
                    quote! {
                        #(#names: impl ::core::convert::Into<#types>),*
                    },
                    quote! { (#(::core::convert::Into::into(#names)),*) },
                )
            } else {
                (
                    quote! {
                        #(#names: #types),*
                    },
                    quote! { (#(#names),*) },
                )
            };
        }

        if self.into {
            (
                quote! { #field_name: impl ::core::convert::Into<#ty> },
                quote! { ::core::convert::Into::into(#field_name) },
            )
        } else {
            (quote! { #field_name: #ty }, field_name.to_token_stream())
        }
    }

    fn parse(
        input: syn::parse::ParseStream,
        builder_attr: &BuilderAttr,
        field: &Field,
    ) -> syn::Result<Self> {
        let mut out = FieldAttr::default();
        let field_ident = field.ident.as_ref().unwrap();

        while input.peek(syn::Ident) {
            let ident: Ident = input.parse()?;
            match Attribute::parse(&ident)? {
                Attribute::Default => {
                    if out.default.is_some() {
                        bail!(ident.span() => "`default` may only be used once");
                    }

                    if out.repeat.is_some() {
                        bail!(ident.span() => "`default` cannot be added with `repeat`");
                    }

                    let value: Option<Expr> = if input.peek(Token![=]) {
                        let _: Token![=] = input.parse()?;
                        let s: LitStr = input.parse()?;
                        Some(s.parse()?)
                    } else {
                        None
                    };

                    out.default = Some(value)
                }
                Attribute::Into => {
                    if out.into {
                        bail!(ident.span() => "`into` may only be used once");
                    }

                    if out.adapter.is_some() {
                        bail!(ident.span() => "`into` cannot be added with `adapter`");
                    }

                    out.into = true
                }
                Attribute::Repeat => {
                    if out.repeat.is_some() {
                        bail!(ident.span() => "`repeat` may only be used once");
                    }

                    if out.default.is_some() {
                        bail!(ident.span() => "`repeat` cannot be added with `default`");
                    }

                    let (inner, len, array) = if input.peek(Token![=]) {
                        if let Type::Array(_) = &field.ty {
                            bail!(ident.span() => "`repeat` cannot be used with a type on arrays");
                        }

                        let _: Token![=] = input.parse()?;
                        let s: Type = input.parse()?;
                        (s, Len::None, false)
                    } else {
                        let Some(inner) = get_single_generic(&field.ty, None) else {
                            bail!(field.ty.span() => "Inner type must be specified to repeat on type without generics");
                        };

                        if let Type::Array(array) = &field.ty {
                            let len = &array.len;
                            let pattern: Pat = parse_quote! { #len };

                            let len = if builder_attr.kind == Kind::TypeState {
                                let len = Len::try_from(pattern)?;
                                if let Len::Range { .. } = len {
                                    unreachable!("Arrays can't have ranges for length");
                                }
                                len
                            } else {
                                let error = format_ident!(
                                    "Range{}",
                                    field_ident.to_string().to_case(Case::Pascal)
                                );
                                Len::Raw { pattern, error }
                            };

                            (inner.clone(), len, true)
                        } else {
                            (inner.clone(), Len::None, false)
                        }
                    };

                    out.repeat = Some(Repeat {
                        inner_ty: inner,
                        len,
                        array,
                    });
                }
                Attribute::RepeatN => {
                    let Some(rep) = &mut out.repeat else {
                        bail!(ident.span() => "`repeat_n` may only be used with `repeat`");
                    };

                    if rep.array {
                        bail!(ident.span() => "`repeat_n` may not be used on arrays");
                    }

                    if rep.len.is_some() {
                        bail!(ident.span() => "`repeat_n` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;

                    let pat = Pat::parse_multi(input)?;

                    rep.len = if builder_attr.kind == Kind::TypeState {
                        Len::try_from(pat)?
                    } else {
                        let err =
                            format_ident!("Range{}", field_ident.to_string().to_case(Case::Pascal));
                        Len::Raw {
                            pattern: pat,
                            error: err,
                        }
                    };
                }
                Attribute::Rename => {
                    if out.rename.is_some() {
                        bail!(ident.span() => "`rename` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    let s: LitStr = input.parse()?;

                    out.rename = Some(s.parse()?);
                }
                Attribute::SkipPrefix => {
                    if out.skip_prefix {
                        bail!(ident.span() => "`skip_prefix` may only be used once");
                    }
                    out.skip_prefix = true;
                }
                Attribute::SkipSuffix => {
                    if out.skip_suffix {
                        bail!(ident.span() => "`skip_suffix` may only be used once");
                    }
                    out.skip_suffix = true;
                }
                Attribute::Tuple => {
                    if out.tuple.is_some() {
                        bail!(ident.span() => "`tuple` may only be used once");
                    }

                    if out.adapter.is_some() {
                        bail!(ident.span() => "`tuple` cannot be added with `adapter`");
                    }

                    let tuple = match &field.ty {
                        Type::Tuple(tuple) => tuple,
                        _ => match &out.repeat {
                            Some(Repeat {
                                inner_ty: Type::Tuple(tuple),
                                ..
                            }) => tuple,
                            _ => {
                                bail!(ident.span() => "`tuple` may only be used on fields that are tuples");
                            }
                        },
                    };

                    if input.peek(Paren) {
                        let content;
                        let paren = parenthesized!(content in input);
                        let idents = content.parse_terminated(Ident::parse, Token![,])?;

                        match tuple.elems.len().cmp(&idents.len()) {
                            std::cmp::Ordering::Less => {
                                bail!(paren.span.join() => "More names than elements in tuple")
                            }
                            std::cmp::Ordering::Equal => {}
                            std::cmp::Ordering::Greater => {
                                bail!(paren.span.join() => "Fewer names than elements in tuple")
                            }
                        }

                        out.tuple = Some(Some(idents.into_iter().collect()))
                    } else {
                        out.tuple = Some(None)
                    }
                }
                Attribute::Adapter => {
                    if out.adapter.is_some() {
                        bail!(ident.span() => "`adapter` may only be used once");
                    }

                    if out.tuple.is_some() {
                        bail!(ident.span() => "`adapter` cannot be added with `tuple`");
                    }

                    if out.into {
                        bail!(ident.span() => "`adapter` cannot be added with `into`");
                    }

                    let la = input.lookahead1();
                    let adapters: Adapter = if la.peek(Paren) {
                        let a;
                        parenthesized!(a in input);
                        a.parse()?
                    } else if la.peek(Token![=]) {
                        let _: Token![=] = input.parse()?;
                        input.parse()?
                    } else {
                        return Err(la.error());
                    };

                    out.adapter = Some(adapters);
                }
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else {
                break;
            }
        }

        Ok(out)
    }
}
