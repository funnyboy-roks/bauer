use convert_case::{Case, Casing};
use quote::format_ident;
use syn::{
    Expr, ExprRange, Field, Ident, LitStr, Meta, Token, Type, Visibility, parenthesized,
    parse::{Parse, ParseStream},
    spanned::Spanned,
    token::Paren,
};

use crate::get_single_generic;

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
}

impl Attribute {
    const ALL: [Self; 8] = [
        Self::Default,
        Self::Into,
        Self::Repeat,
        Self::RepeatN,
        Self::Rename,
        Self::SkipPrefix,
        Self::SkipSuffix,
        Self::Tuple,
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

pub struct BuilderField {
    pub ident: Ident,
    #[allow(unused)]
    pub vis: Visibility,
    pub ty: Type,
    pub attr: FieldAttr,
    pub missing_err: Option<Ident>,
    pub wrapped_option: bool,
    pub doc: Vec<syn::Attribute>,
}

pub struct Repeat {
    pub inner_ty: Type,
    pub len: Option<(ExprRange, Ident)>,
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
}

impl FieldAttr {
    fn parse(input: syn::parse::ParseStream, field: &Field) -> syn::Result<Self> {
        let mut out = FieldAttr::default();
        let field_ident = field.ident.as_ref().unwrap();

        while input.peek(syn::Ident) {
            let ident: Ident = input.parse()?;
            match Attribute::parse(&ident)? {
                Attribute::Default => {
                    if out.default.is_some() {
                        bail!(ident.span() => "`default` may only be used once.");
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
                        bail!(ident.span() => "`into` may only be used once.");
                    }

                    out.into = true
                }
                Attribute::Repeat => {
                    if out.repeat.is_some() {
                        bail!(ident.span() => "`repeat` may only be used once.");
                    }

                    if out.default.is_some() {
                        bail!(ident.span() => "`repeat` cannot be added with `default`");
                    }

                    let Some(inner) = get_single_generic(&field.ty, None) else {
                        bail!(field.ty.span() => "Cannot repeat on value with no generics");
                    };

                    out.repeat = Some(Repeat {
                        inner_ty: inner.clone(),
                        len: None,
                    });
                }
                Attribute::RepeatN => {
                    let Some(rep) = &mut out.repeat else {
                        bail!(ident.span() => "`repeat_n` may only be used with `repeat`");
                    };

                    if rep.len.is_some() {
                        bail!(ident.span() => "`repeat_n` may only be used once.");
                    }

                    let _: Token![=] = input.parse()?;
                    let mut ident =
                        format_ident!("Range{}", field_ident.to_string().to_case(Case::Pascal));
                    ident.set_span(ident.span());
                    rep.len = Some((input.parse()?, ident));
                }
                Attribute::Rename => {
                    if out.rename.is_some() {
                        bail!(ident.span() => "`rename` may only be used once.");
                    }

                    let _: Token![=] = input.parse()?;
                    let s: LitStr = input.parse()?;

                    out.rename = Some(s.parse()?);
                }
                Attribute::SkipPrefix => {
                    if out.skip_prefix {
                        bail!(ident.span() => "`skip_prefix` may only be used once.");
                    }
                    out.skip_prefix = true;
                }
                Attribute::SkipSuffix => {
                    if out.skip_suffix {
                        bail!(ident.span() => "`skip_suffix` may only be used once.");
                    }
                    out.skip_suffix = true;
                }
                Attribute::Tuple => {
                    if out.tuple.is_some() {
                        bail!(ident.span() => "`tuple` may only be used once.");
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

impl TryFrom<&Field> for BuilderField {
    type Error = syn::Error;

    fn try_from(value: &Field) -> Result<Self, Self::Error> {
        let ident = value.ident.as_ref().expect("We only support named fields");
        let attr: FieldAttr =
            if let Some(builder_attr) = value.attrs.iter().find(|a| a.path().is_ident("builder")) {
                builder_attr.parse_args_with(|input: ParseStream| FieldAttr::parse(input, value))?
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
            vis: value.vis.clone(),
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
        })
    }
}
