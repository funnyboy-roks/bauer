use quote::format_ident;
use strum::{AsRefStr, IntoStaticStr, VariantArray};
use syn::{
    ExprClosure, Ident, LitStr, Token, Type, ext::IdentExt, parse::ParseStream, spanned::Spanned,
};

use crate::util::parse::{parethesised_or_braced, parse_attributes, parse_docs};

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
    #[allow(clippy::enum_variant_names)]
    Attributes,
    Doc,
    Rename,
    Map,
}

impl Attribute {
    fn matches(self, ident: &Ident) -> bool {
        if ident == self.as_ref() {
            return true;
        }

        match self {
            Self::Attributes => ident == "attribute",
            Self::Doc => ident == "docs",
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
pub struct BuildFnAttr {
    pub is_builder: bool,
    pub attributes: Vec<syn::Attribute>,
    pub name: Ident,
    pub mapper: Option<(ExprClosure, Type)>,
}

impl BuildFnAttr {
    pub fn default_build() -> Self {
        Self {
            is_builder: false,
            attributes: Default::default(),
            name: format_ident!("build"),
            mapper: None,
        }
    }

    pub fn default_builder() -> Self {
        Self {
            is_builder: true,
            attributes: Default::default(),
            name: format_ident!("builder"),
            mapper: None,
        }
    }
}

impl BuildFnAttr {
    pub fn parse(&mut self, input: ParseStream) -> syn::Result<()> {
        let mut rename_set = false;

        while input.peek(Ident::peek_any) {
            let ident = Ident::parse_any(input)?;
            match Attribute::parse(&ident)? {
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
                Attribute::Rename => {
                    if rename_set {
                        bail!(ident.span() => "`rename` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    let s: LitStr = input.parse()?;

                    rename_set = true;
                    self.name = s.parse()?;
                }
                Attribute::Map => {
                    if self.is_builder {
                        bail!(ident.span() => "`map` may not be specified on builder_fn");
                    }

                    let _: Token![=] = input.parse()?;
                    let closure: ExprClosure = input.parse()?;

                    if closure.inputs.len() != 1 {
                        bail!(closure.span() => "`map` closure must take one input");
                    }

                    match closure.output {
                        syn::ReturnType::Default => {
                            bail!(closure.span() => "`map` closure must specify a return type")
                        }
                        syn::ReturnType::Type(_, ref ty) => {
                            self.mapper = Some((closure.clone(), (**ty).clone()))
                        }
                    }
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
