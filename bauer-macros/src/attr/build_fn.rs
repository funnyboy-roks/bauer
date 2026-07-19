use quote::format_ident;
use strum::{AsRefStr, IntoStaticStr, VariantArray};
use syn::{
    Expr, ExprClosure, Ident, LitStr, Token, Type, Visibility, ext::IdentExt, parse::ParseStream,
    spanned::Spanned,
};

use crate::util::parse::{parethesised_or_braced, parse_attributes, parse_docs};

use super::builder::BuilderAttr;

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
#[repr(usize)]
enum Attribute {
    #[allow(clippy::enum_variant_names)]
    Attributes = 0,
    Doc,
    Rename,
    Map,
    Visibility,
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

    const fn index(self) -> usize {
        self as usize
    }

    const fn single_use(self) -> bool {
        match self {
            Attribute::Attributes => false,
            Attribute::Doc => false,
            Attribute::Rename => true,
            Attribute::Map => true,
            Attribute::Visibility => true,
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
    /// None - Inherit from builder
    /// Some(vis) - use that visibility
    vis: Option<Visibility>,
    pub is_builder: bool,
    pub attributes: Vec<syn::Attribute>,
    pub name: Ident,
    pub mapper: Option<(Ident, Type, Expr)>,
    pub set_fields: [bool; const { Attribute::VARIANTS.len() }],
}

impl BuildFnAttr {
    pub fn default_build() -> Self {
        Self {
            vis: None,
            is_builder: false,
            attributes: Default::default(),
            name: format_ident!("build"),
            mapper: None,
            set_fields: Default::default(),
        }
    }

    pub fn default_builder() -> Self {
        Self {
            vis: None,
            is_builder: true,
            attributes: Default::default(),
            name: format_ident!("builder"),
            mapper: None,
            set_fields: Default::default(),
        }
    }

    pub fn vis<'a>(&'a self, builder_attr: &'a BuilderAttr) -> &'a Visibility {
        self.vis.as_ref().unwrap_or(&builder_attr.vis)
    }
}

impl BuildFnAttr {
    pub fn parse(&mut self, input: ParseStream) -> syn::Result<()> {
        while input.peek(Ident::peek_any) {
            let ident = Ident::parse_any(input)?;
            let attr = Attribute::parse(&ident)?;

            if self.set_fields[attr.index()] && attr.single_use() {
                bail!(ident.span() => "`{}` may only be used once", <&str>::from(attr));
            }
            self.set_fields[attr.index()] = true;

            match attr {
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
                    let _: Token![=] = input.parse()?;
                    let s: LitStr = input.parse()?;
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
                    let ident = match &closure.inputs[0] {
                        syn::Pat::Ident(p) => p.ident.clone(),
                        i => bail!(i.span() => "`map` input must be an identifier"),
                    };

                    match closure.output {
                        syn::ReturnType::Default => {
                            bail!(closure.span() => "`map` closure must specify a return type")
                        }
                        syn::ReturnType::Type(_, ty) => {
                            self.mapper = Some((ident, *ty, *closure.body))
                        }
                    }
                }
                Attribute::Visibility => {
                    let _: Token![=] = input.parse()?;
                    self.vis = Some(input.parse()?);
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
