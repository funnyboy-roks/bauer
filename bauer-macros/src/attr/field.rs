use std::ops::Range;

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use strum::{AsRefStr, IntoStaticStr, VariantArray};
use syn::{
    Expr, ExprClosure, Field, Ident, Index, LitStr, Pat, Path, Token, TraitBound, Type, braced,
    parenthesized,
    parse::{Parse, ParseStream},
    parse_quote, parse_quote_spanned,
    spanned::Spanned,
    token::{Brace, Paren},
};

use crate::{
    BuilderAttr, Kind,
    util::{
        escape_ident,
        parse::{parse_attributes, parse_docs},
    },
};

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

#[derive(Clone, Copy, VariantArray, IntoStaticStr, AsRefStr, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
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
    #[allow(clippy::enum_variant_names)]
    Attributes,
    Doc,
    Collector,
    Skip,
}

impl Attribute {
    fn as_str(self) -> &'static str {
        self.into()
    }
}

impl Attribute {
    fn matches(self, ident: &Ident) -> bool {
        if ident == self.as_ref() {
            return true;
        }

        match self {
            Self::Attributes => ident == "attribute",
            _ => false,
        }
    }

    fn parse(ident: &Ident) -> syn::Result<Self> {
        Self::VARIANTS
            .iter()
            .copied()
            .find(|e| e.matches(ident))
            .ok_or_else(|| {
                syn::Error::new(
                    ident.span(),
                    format!(
                        "Unknown attribute '{}'.  Valid attribute are: '{}'",
                        ident,
                        Self::VARIANTS
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            })
    }
}

#[derive(Debug)]
pub struct FieldIdents {
    pub pascal: Ident,
    pub set: Ident,
    pub count: Ident,
}

impl FieldIdents {
    fn new(struct_name: &Ident, ident: &Ident) -> Self {
        let name = ident.to_string();
        let name = name.trim_start_matches("r#");
        let pascal = Ident::new(&name.to_case(Case::Pascal), ident.span());
        Self {
            set: format_ident!("{}_{}_Set", struct_name, pascal, span = pascal.span()),
            count: format_ident!("{}_{}_Count", struct_name, pascal, span = pascal.span()),
            pascal,
        }
    }
}

#[derive(Debug)]
pub struct BuilderField {
    pub ident: Ident,
    pub ty: Type,
    pub attr: FieldAttr,
    pub missing_err: Option<Ident>,
    pub wrapped_option: bool,
    pub idents: FieldIdents,
    pub tuple_index: usize,
}

impl BuilderField {
    pub fn should_skip(&self) -> bool {
        self.attr.skip.is_set()
    }

    pub fn skipped_field_value(&self) -> Option<TokenStream> {
        let value = match &self.attr.skip {
            Skip::None => return None,
            Skip::Default { ident } => quote_spanned! {ident.span()=>
                ::core::default::Default::default()
            },
            Skip::Expr { ident, expr } => quote_spanned! {ident.span()=>
                #expr
            },
        };

        Some(quote! {{
            #value
        }})
    }

    pub fn wrapped_type(&self) -> Type {
        if self.wrapped_option {
            let ty = &self.ty;
            parse_quote! { ::core::option::Option<#ty> }
        } else {
            self.ty.clone()
        }
    }

    pub fn tuple_index(&self) -> syn::Index {
        syn::Index {
            index: self.tuple_index as _,
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

        escape_ident(format_ident!(
            "{}{}{}",
            prefix,
            ident,
            suffix,
            span = ident.span()
        ))
    }

    pub(crate) fn function(&self, builder_attr: &BuilderAttr, inner: &Ident) -> TokenStream {
        let field_name = &self.ident;

        let ty = self.arg_ty();
        let fn_ident = self.function_ident(builder_attr);
        let (args, value) = self.attr.to_args_and_value(ty, field_name);
        let self_param = builder_attr.self_param();
        let return_type = builder_attr.return_type();
        let builder_vis = &builder_attr.vis;

        let field_i = self.tuple_index();

        let setter = if self.attr.repeat.is_some() {
            quote! { let _ = self.#inner.#field_i.push(value) }
        } else {
            quote! { self.#inner.#field_i = Some(value) }
        };

        let attributes = &self.attr.attributes;
        let konst = builder_attr.konst_kw();

        quote! {
            #(#attributes)*
            #[must_use = "The builder doesn't construct its type until `.build()` is called"]
            #builder_vis #konst fn #fn_ident(#self_param, #args) -> #return_type {
                let value: #ty = #value;
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
        tuple_index: &mut usize,
    ) -> syn::Result<Self> {
        let ident = value.ident.as_ref().expect("We only support named fields");

        let (ty, wrapped_option) = if let Some(ty) = get_single_generic(&value.ty, Some("Option")) {
            (ty, true)
        } else {
            (&value.ty, false)
        };

        let mut attr: FieldAttr =
            if let Some(attr) = value.attrs.iter().find(|a| a.path().is_ident("builder")) {
                attr.parse_args_with(|input: ParseStream| {
                    FieldAttr::parse(input, builder_attr, value, wrapped_option)
                })?
            } else {
                FieldAttr::default()
            };

        if !attr.attributes.iter().any(|a| a.path().is_ident("doc")) {
            attr.attributes.reserve(value.attrs.len());
            value
                .attrs
                .iter()
                .filter(|a| a.path().is_ident("doc"))
                .cloned()
                .for_each(|a| attr.attributes.push(a))
        }

        let this_tuple_index = *tuple_index;

        if !attr.skip.is_set() {
            *tuple_index += 1;
        }

        Ok(BuilderField {
            ident: ident.clone(),
            ty: ty.clone(),
            missing_err: if attr.default.is_none() && attr.repeat.is_none() && !wrapped_option {
                let mut ident = format_ident!(
                    "Missing{}",
                    ident
                        .to_string()
                        .trim_start_matches("r#")
                        .to_case(Case::Pascal)
                );
                ident.set_span(value.ident.as_ref().unwrap().span());
                Some(ident)
            } else {
                None
            },
            attr,
            wrapped_option,
            idents: FieldIdents::new(struct_name, ident),
            tuple_index: this_tuple_index,
        })
    }
}

#[derive(Clone, Debug)]
pub enum Len {
    /// No length specified
    None,
    /// Length specified, but parsing was not necessary (not type-state builder)
    Raw {
        pattern: Pat,
        error: Ident,
    },
    Int {
        len: usize,
    },
    Range {
        start: usize,
        end: Option<usize>,
        inclusive: bool,
        pat: syn::Pat,
    },
}

impl Len {
    pub fn is_some(&self) -> bool {
        !self.is_none()
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    fn range(&self) -> Option<Range<usize>> {
        match self {
            Len::None => None,
            Len::Raw { .. } => None,
            &Len::Int { len } => Some(len..len + 1),
            &Len::Range {
                start, end: None, ..
            } => Some(start..usize::MAX),
            &Len::Range {
                start,
                end: Some(end),
                inclusive,
                ..
            } => Some(start..end + usize::from(inclusive)),
        }
    }

    fn expanded_tuple(base: Type, depth: usize) -> Type {
        let mut out = base;
        for _ in 0..depth {
            out = parse_quote! { (#out, ()) };
        }
        out
    }

    pub fn to_trait(
        &self,
        krate: &Ident,
        wrapper: &Ident,
        out: &mut TokenStream,
    ) -> syn::Result<TraitBound> {
        match self {
            Len::None => unreachable!("Len::into_trait called on None"),
            Len::Raw { .. } => unreachable!("Len::into_trait called on Raw"),
            &Len::Int { len } => {
                let expanded = Self::expanded_tuple(parse_quote! { () }, len);
                out.extend(quote! {
                    impl ::#krate::__private::sealed::Sealed for #wrapper<#expanded> {}
                    impl ::#krate::state::Eq<#len> for #wrapper<#expanded> {}
                });
                let len = Index::from(len); // remove the `usize` suffix
                Ok(parse_quote! {
                    ::#krate::state::Eq<#len>
                })
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

                let trait_: Path = if *inclusive {
                    parse_quote! {
                        ::#krate::state::RangeInclusive
                    }
                } else {
                    parse_quote! {
                        ::#krate::state::RangeExclusive
                    }
                };

                let start = Index::from(*start); // remove the `usize` suffix
                let end = Index::from(*end); // remove the `usize` suffix
                for i in range {
                    let expanded = Self::expanded_tuple(parse_quote! { () }, i);
                    out.extend(quote! {
                        impl ::#krate::__private::sealed::Sealed for #wrapper<#expanded> {}
                        impl #trait_<#start, #end> for #wrapper<#expanded> {}
                    });
                }

                Ok(parse_quote! {
                    #trait_<#start, #end>
                })
            }
            Len::Range {
                start, end: None, ..
            } => {
                let expanded = Self::expanded_tuple(parse_quote! { T }, *start);
                let start = Index::from(*start); // remove the `usize` suffix
                out.extend(quote! {
                    #[allow(non_camel_case_types)]
                    impl<T> ::#krate::__private::sealed::Sealed for #wrapper<#expanded> {}
                    impl<T> ::#krate::state::AtLeast<#start> for #wrapper<#expanded> {}
                });
                Ok(parse_quote! {
                    ::#krate::state::AtLeast<#start>
                })
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
                Len::Int { len }
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

#[derive(Debug)]
#[allow(clippy::large_enum_variant)] // not relly important
pub enum Collector {
    FromIterator {
        inner_ty: Type,
    },
    Custom {
        collector: Expr,
        inner_ty: Type,
        field_ty: Type,
    },
}

impl Collector {
    fn collector_fn(&self) -> TokenStream {
        match self {
            Collector::FromIterator { inner_ty } => {
                quote_spanned! { inner_ty.span()=>
                    let collector = ::core::iter::FromIterator::from_iter;
                }
            }
            Collector::Custom {
                collector,
                inner_ty,
                field_ty,
            } => {
                quote_spanned! { collector.span()=>
                    fn collector(iter: impl ::core::iter::ExactSizeIterator<Item = #inner_ty>) -> #field_ty {
                        let collector = #collector;
                        collector(iter)
                    }
                }
            }
        }
    }

    fn input_span(&self) -> Span {
        match self {
            Collector::FromIterator { inner_ty } => inner_ty.span(),
            Collector::Custom { collector, .. } => collector.span(),
        }
    }

    pub fn collect(&self, inner: Expr) -> TokenStream {
        let collector_fn = self.collector_fn();
        quote_spanned! {self.input_span()=>{
            #collector_fn
            collector(#inner)
        }}
    }
}

#[derive(Debug)]
pub struct Repeat {
    pub inner_ty: Type,
    pub len: Len,
    pub array: bool,
    pub collector: Collector,
}

#[derive(Debug)]
pub enum DefaultAttr {
    /// Default value is not specified, so use Default::default
    Default { ident: Ident },
    /// Use user-defined expression as default
    Custom { ident: Ident, expr: Expr },
}

impl DefaultAttr {
    pub fn to_value(&self, into_span: Option<Span>) -> Expr {
        match (self, into_span) {
            (DefaultAttr::Default { ident }, _) => parse_quote_spanned! {ident.span()=>
                ::core::default::Default::default()
            },
            (DefaultAttr::Custom { ident: _, expr }, Some(span)) => parse_quote_spanned! {span=>
                ::core::convert::Into::into(#expr)
            },
            (DefaultAttr::Custom { ident, expr }, None) => parse_quote_spanned! {ident.span()=>
                #expr
            },
        }
    }
}

#[derive(Debug)]
pub struct Adapter {
    args: Vec<(Ident, Type)>,
    expr: Expr,
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

#[derive(Default, Debug)]
pub enum Skip {
    #[default]
    None,
    Default {
        ident: Ident,
    },
    Expr {
        ident: Ident,
        expr: Expr,
    },
}

impl Skip {
    pub fn is_set(&self) -> bool {
        !matches!(self, Self::None)
    }

    pub fn ident(&self) -> Option<&Ident> {
        match self {
            Skip::None => None,
            Skip::Default { ident } => Some(ident),
            Skip::Expr { ident, .. } => Some(ident),
        }
    }
}

#[derive(Default, Debug)]
pub struct FieldAttr {
    pub skip: Skip,
    pub default: Option<DefaultAttr>,
    pub into: Option<Span>,
    pub repeat: Option<Repeat>,
    pub rename: Option<Ident>,
    pub skip_prefix: bool,
    pub skip_suffix: bool,
    /// Some(Some(names)) -> tuple argument use `names` for names
    /// Some(None)        -> tuple argument use `field_N` for names
    /// None              -> tuple is passed as a value
    pub tuple: Option<Option<Vec<Ident>>>,
    pub adapter: Option<Adapter>,
    pub attributes: Vec<syn::Attribute>,
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

            return if let Some(span) = self.into {
                (
                    quote_spanned! {span=> #(#names: impl ::core::convert::Into<#types>),* },
                    quote_spanned! {span=> (#(::core::convert::Into::into(#names)),*) },
                )
            } else {
                (quote! { #(#names: #types),* }, quote! { (#(#names),*) })
            };
        }

        if let Some(span) = self.into {
            (
                quote_spanned! {span=> #field_name: impl ::core::convert::Into<#ty> },
                quote_spanned! {span=> ::core::convert::Into::into(#field_name) },
            )
        } else {
            (quote! { #field_name: #ty }, field_name.to_token_stream())
        }
    }

    fn parse(
        input: syn::parse::ParseStream,
        builder_attr: &BuilderAttr,
        field: &Field,
        wrapped_option: bool,
    ) -> syn::Result<Self> {
        let mut out = FieldAttr::default();
        let field_ident = field.ident.as_ref().unwrap();

        let mut n_attr = 0;
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

                    if wrapped_option {
                        bail!(ident.span() => "`default` may not be used on `Option` fields");
                    }

                    let value = if input.peek(Token![=]) {
                        let _: Token![=] = input.parse()?;
                        let s: LitStr = input.parse()?;
                        DefaultAttr::Custom {
                            ident,
                            expr: s.parse()?,
                        }
                    } else {
                        if builder_attr.konst {
                            bail!(ident.span() => "`default` may not be used without a value on const builders");
                        }

                        DefaultAttr::Default { ident }
                    };

                    out.default = Some(value)
                }
                Attribute::Into => {
                    if out.into.is_some() {
                        bail!(ident.span() => "`into` may only be used once");
                    }

                    if out.adapter.is_some() {
                        bail!(ident.span() => "`into` cannot be added with `adapter`");
                    }

                    if builder_attr.konst {
                        bail!(ident.span() => "`into` may not be used on const builders");
                    }

                    out.into = Some(ident.span());
                }
                Attribute::Repeat => {
                    if out.repeat.is_some() {
                        bail!(ident.span() => "`repeat` may only be used once");
                    }

                    if out.default.is_some() {
                        bail!(ident.span() => "`repeat` cannot be added with `default`");
                    }

                    if builder_attr.konst && !matches!(field.ty, Type::Array(_)) {
                        bail!(ident.span() => "`repeat` may only be used for arrays on const builders");
                    }

                    let (inner_ty, len, array) = if input.peek(Token![=]) {
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
                        inner_ty: inner_ty.clone(),
                        len,
                        array,
                        collector: Collector::FromIterator { inner_ty },
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

                    if builder_attr.konst {
                        bail!(ident.span() => "`repeat_n` may not be used on const builders");
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

                    if out.into.is_some() {
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
                Attribute::Attributes => {
                    let attrs;

                    let la = input.lookahead1();
                    if la.peek(Paren) {
                        parenthesized!(attrs in input);
                    } else if la.peek(Brace) {
                        braced!(attrs in input);
                    } else {
                        return Err(la.error());
                    }

                    if !attrs.is_empty() {
                        parse_attributes(&attrs, &mut out.attributes)?;
                    }
                }
                Attribute::Doc => {
                    let attrs;

                    let la = input.lookahead1();
                    if la.peek(Paren) {
                        parenthesized!(attrs in input);
                    } else if la.peek(Brace) {
                        braced!(attrs in input);
                    } else {
                        return Err(la.error());
                    }

                    if !attrs.is_empty() {
                        parse_docs(&attrs, ident.span(), &mut out.attributes)?;
                    }
                }
                Attribute::Collector => {
                    let Some(repeat) = &mut out.repeat else {
                        bail!(ident.span() => "`collector` may only be used with `repeat`");
                    };

                    if !matches!(repeat.collector, Collector::FromIterator { .. }) {
                        bail!(ident.span() => "`collector` may only be used once");
                    };

                    if repeat.array {
                        bail!(ident.span() => "`collector` may not be used on arrays");
                    }

                    let _: Token![=] = input.parse()?;
                    let collector: Expr = input.parse()?;

                    repeat.collector = Collector::Custom {
                        collector,
                        inner_ty: repeat.inner_ty.clone(),
                        field_ty: field.ty.clone(),
                    };
                }
                Attribute::Skip => {
                    if out.skip.is_set() {
                        bail!(ident.span() => "`skip` may only be used once");
                    }

                    out.skip = if input.peek(Token![=]) {
                        let _: Token![=] = input.parse()?;
                        Skip::Expr {
                            ident,
                            expr: input.parse()?,
                        }
                    } else {
                        Skip::Default { ident }
                    };
                }
            }
            n_attr += 1;

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else {
                break;
            }
        }

        // validate the structure
        if n_attr != 1 {
            if let Some(ident) = out.skip.ident() {
                bail!(
                    ident.span() =>
                    "`skip` may not be used with any other attributes"
                );
            }
        }

        Ok(out)
    }
}
