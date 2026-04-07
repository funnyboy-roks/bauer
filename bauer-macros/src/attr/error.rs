use quote::format_ident;
use strum::{AsRefStr, IntoStaticStr, VariantArray};
use syn::{Ident, LitStr, Token, ext::IdentExt, parse::ParseStream};

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
    Force,
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

#[derive(Debug, Clone, Default)]
pub struct ErrorAttr {
    pub attributes: Vec<syn::Attribute>,
    pub rename: Option<Ident>,
    pub force: bool,
}

impl ErrorAttr {
    pub fn name(&self, struct_name: &Ident) -> Ident {
        if let Some(rename) = &self.rename {
            rename.clone()
        } else {
            format_ident!("{}BuildError", struct_name)
        }
    }

    pub fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut out = Self::default();

        while input.peek(Ident::peek_any) {
            let ident = Ident::parse_any(input)?;
            match Attribute::parse(&ident)? {
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
                Attribute::Rename => {
                    if out.rename.is_some() {
                        bail!(ident.span() => "`rename` may only be used once");
                    }

                    let _: Token![=] = input.parse()?;
                    let s: LitStr = input.parse()?;

                    out.rename = Some(s.parse()?);
                }
                Attribute::Force => {
                    if out.force {
                        bail!(ident.span() => "`force` may only be used once");
                    }

                    out.force = true;
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
