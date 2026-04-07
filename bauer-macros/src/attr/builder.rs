use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use strum::{AsRefStr, IntoStaticStr, VariantArray};
use syn::{
    Ident, ItemConst, LitStr, Token, Visibility,
    ext::IdentExt,
    parse::{Parse, ParseStream},
    parse_quote,
};

use crate::{
    attr::{build_fn::BuildFnAttr, error::ErrorAttr},
    util::{
        OptionalToken,
        parse::{parethesised_or_braced, parse_attributes, parse_docs},
    },
};

macro_rules! bail {
    ($span: expr => $message: literal $(, $args: expr)*$(,)?) => {
        return Err(syn::Error::new(
            $span,
            format!($message, $($args),*),
        ))
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    #[default]
    Owned,
    Borrowed,
    TypeState,
}

impl FromStr for Kind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "owned" => Ok(Self::Owned),
            "borrowed" => Ok(Self::Borrowed),
            "type-state" => Ok(Self::TypeState),
            _ => Err(format!(
                "Unknown kind \"{}\".  Valid kinds are: \"owned\", \"borrowed\", \"type-state\"",
                s
            )),
        }
    }
}

impl Parse for Kind {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let s: LitStr = input.parse()?;
        match Kind::from_str(&s.value()) {
            Ok(v) => Ok(v),
            Err(e) => Err(syn::Error::new(s.span(), e)),
        }
    }
}

#[derive(Clone, Copy, VariantArray, IntoStaticStr, AsRefStr, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
enum Attribute {
    Kind,
    Prefix,
    Suffix,
    Visibility,
    Crate,
    Const,
    #[allow(clippy::enum_variant_names)]
    Attributes,
    Doc,
    BuildFn,
    BuilderFn,
    Error,
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
                            .map(<&str>::from)
                            .collect::<Vec<_>>()
                            .join(", ")
                    ),
                )
            })
    }
}

#[derive(Debug, Clone)]
pub struct BuilderAttr {
    pub kind: Kind,
    pub prefix: String,
    pub suffix: String,
    pub vis: Visibility,
    pub krate: Ident,
    pub konst: bool,
    pub attributes: Vec<syn::Attribute>,
    pub build_fn: BuildFnAttr,
    pub builder_fn: BuildFnAttr,
    pub error: ErrorAttr,
}

impl BuilderAttr {
    pub fn new(vis: Visibility) -> Self {
        Self {
            kind: Default::default(),
            prefix: Default::default(),
            suffix: Default::default(),
            vis,
            krate: format_ident!("bauer"),
            konst: false,
            attributes: Default::default(),
            build_fn: BuildFnAttr::default_build(),
            builder_fn: BuildFnAttr::default_builder(),
            error: Default::default(),
        }
    }

    pub fn assert_crate(&self) -> ItemConst {
        let private_module = self.private_module();
        parse_quote! {
            const _: () = {
                #[allow(unused)]
                pub use #private_module as _;
            };
        }
    }

    pub fn private_module(&self) -> syn::Path {
        let krate = &self.krate;
        parse_quote! { ::#krate::__private }
    }

    pub fn konst_kw(&self) -> OptionalToken<Token![const]> {
        if self.konst {
            OptionalToken(Some(<Token![const]>::default()))
        } else {
            OptionalToken(None)
        }
    }

    pub fn self_param(&self) -> TokenStream {
        match self.kind {
            Kind::Owned => quote! { mut self },
            Kind::Borrowed => quote! { &mut self },
            Kind::TypeState => quote! { self },
        }
    }

    pub fn return_type(&self) -> TokenStream {
        match self.kind {
            Kind::Owned => quote! { Self },
            Kind::Borrowed => quote! { &mut Self },
            Kind::TypeState => {
                panic!("This function should not be called on Kind::TypeState")
            }
        }
    }

    pub fn parse(input: ParseStream, vis: Visibility) -> syn::Result<Self> {
        let mut out = Self::new(vis);

        let mut kind_set = false;
        let mut prefix_set = false;
        let mut suffix_set = false;
        let mut vis_set = false;
        let mut crate_set = false;
        let mut build_fn_set = false;
        let mut builder_fn_set = false;
        let mut error_set = false;

        while input.peek(Ident::peek_any) {
            let ident = Ident::parse_any(input)?;
            match Attribute::parse(&ident)? {
                Attribute::Kind => {
                    if kind_set {
                        bail!(ident.span() => "`kind` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    out.kind = input.parse()?;
                    kind_set = true;
                }
                Attribute::Prefix => {
                    if prefix_set {
                        bail!(ident.span() => "`prefix` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    out.prefix = input.parse::<LitStr>()?.value();
                    prefix_set = true;
                }
                Attribute::Suffix => {
                    if suffix_set {
                        bail!(ident.span() => "`suffix` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    out.suffix = input.parse::<LitStr>()?.value();
                    suffix_set = true;
                }
                Attribute::Visibility => {
                    if vis_set {
                        bail!(ident.span() => "`visibility` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    out.vis = input.parse()?;
                    vis_set = true;
                }
                Attribute::Crate => {
                    if crate_set {
                        bail!(ident.span() => "`crate` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    out.krate = input.parse()?;
                    crate_set = true;
                }
                Attribute::Const => {
                    if out.konst {
                        bail!(ident.span() => "`const` may only be used once");
                    }

                    out.konst = true;
                }
                Attribute::Attributes => {
                    let attrs = parethesised_or_braced(input)?;

                    if !attrs.is_empty() {
                        parse_attributes(&attrs, &mut out.attributes)?;
                    }
                }
                Attribute::Doc => {
                    let attrs = parethesised_or_braced(input)?;

                    if !attrs.is_empty() {
                        parse_docs(&attrs, ident.span(), &mut out.attributes)?;
                    }
                }
                Attribute::BuildFn => {
                    if build_fn_set {
                        bail!(ident.span() => "`build_fn` may only be used once");
                    }

                    let build_fn = parethesised_or_braced(input)?;
                    out.build_fn.parse(&build_fn)?;

                    build_fn_set = true;
                }
                Attribute::BuilderFn => {
                    if builder_fn_set {
                        bail!(ident.span() => "`builder_fn` may only be used once");
                    }

                    let builder_fn = parethesised_or_braced(input)?;
                    out.builder_fn.parse(&builder_fn)?;

                    builder_fn_set = true;
                }
                Attribute::Error => {
                    if error_set {
                        bail!(ident.span() => "`build_fn` may only be used once");
                    }

                    let error = parethesised_or_braced(input)?;
                    out.error = ErrorAttr::parse(&error)?;

                    error_set = true;
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
