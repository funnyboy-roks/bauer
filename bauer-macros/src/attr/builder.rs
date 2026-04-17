use std::str::FromStr;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use strum::{AsRefStr, IntoStaticStr, VariantArray};
use syn::{
    Ident, ItemConst, LitStr, Token, Type, Visibility,
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

#[derive(Clone, Debug)]
pub(crate) struct On {
    pattern: Type,
    attributes: TokenStream,
}

impl On {
    pub fn apply(&self, field_ty: &Type) -> syn::Result<Option<TokenStream>> {
        use crate::util::pattern::{pattern_match_type, replace};

        pattern_match_type(&self.pattern, field_ty)
            .map(|matches| replace(&matches, self.attributes.clone()))
            .transpose()
    }
}

impl Parse for On {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let pattern: Type = input.parse()?;
        let _arrow: Token![=>] = input.parse()?;
        let attributes: TokenStream = input.parse()?;
        Ok(Self {
            pattern,
            attributes,
        })
    }
}

#[derive(Clone, Copy, VariantArray, IntoStaticStr, AsRefStr, Debug, PartialEq, Eq)]
#[strum(serialize_all = "snake_case")]
#[repr(usize)]
enum Attribute {
    Kind = 0,
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
    On,
}

impl Attribute {
    #[inline]
    const fn single_use(&self) -> bool {
        match self {
            Attribute::Kind => true,
            Attribute::Prefix => true,
            Attribute::Suffix => true,
            Attribute::Visibility => true,
            Attribute::Crate => true,
            Attribute::Const => true,
            Attribute::Attributes => false,
            Attribute::Doc => false,
            Attribute::BuildFn => true,
            Attribute::BuilderFn => true,
            Attribute::Error => true,
            Attribute::On => false,
        }
    }

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
    pub on: Vec<On>,
    pub set_fields: [bool; const { Attribute::VARIANTS.len() }],
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
            on: Default::default(),
            set_fields: Default::default(),
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

    pub fn parse(&mut self, input: ParseStream) -> syn::Result<()> {
        while input.peek(Ident::peek_any) {
            let ident = Ident::parse_any(input)?;
            let attr = Attribute::parse(&ident)?;

            if self.set_fields[attr as usize] && attr.single_use() {
                bail!(ident.span() => "`{}` may only be used once", <&str>::from(attr));
            }
            self.set_fields[attr as usize] = true;

            match Attribute::parse(&ident)? {
                Attribute::Kind => {
                    let _: Token![=] = input.parse()?;
                    self.kind = input.parse()?;
                }
                Attribute::Prefix => {
                    let _: Token![=] = input.parse()?;
                    self.prefix = input.parse::<LitStr>()?.value();
                }
                Attribute::Suffix => {
                    let _: Token![=] = input.parse()?;
                    self.suffix = input.parse::<LitStr>()?.value();
                }
                Attribute::Visibility => {
                    let _: Token![=] = input.parse()?;
                    self.vis = input.parse()?;
                }
                Attribute::Crate => {
                    let _: Token![=] = input.parse()?;
                    self.krate = input.parse()?;
                }
                Attribute::Const => {
                    self.konst = true;
                }
                Attribute::Attributes => {
                    let attrs = parethesised_or_braced(input)?;

                    if !attrs.is_empty() {
                        parse_attributes(&attrs, &mut self.attributes)?;
                    }
                }
                Attribute::Doc => {
                    let attrs = parethesised_or_braced(input)?;

                    if !attrs.is_empty() {
                        parse_docs(&attrs, ident.span(), &mut self.attributes)?;
                    }
                }
                Attribute::BuildFn => {
                    let build_fn = parethesised_or_braced(input)?;
                    self.build_fn.parse(&build_fn)?;
                }
                Attribute::BuilderFn => {
                    let builder_fn = parethesised_or_braced(input)?;
                    self.builder_fn.parse(&builder_fn)?;
                }
                Attribute::Error => {
                    let error = parethesised_or_braced(input)?;
                    self.error = ErrorAttr::parse(&error)?;
                }
                Attribute::On => {
                    let inner = parethesised_or_braced(input)?;
                    self.on.push(inner.parse()?);
                }
            }

            if input.peek(Token![,]) {
                let _: Token![,] = input.parse()?;
            } else {
                break;
            }
        }

        Ok(())
    }
}
