use proc_macro2::{Group, TokenStream, TokenTree};
use quote::ToTokens;
use syn::{Ident, Token, bracketed, parse::Parse, parse_macro_input, punctuated::Punctuated};

/// Place all tokens within the parenthesis before the item
#[proc_macro_attribute]
pub fn pre(
    mut attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    attr.extend(item);
    attr
}

/// Blank attribute that provides #[my_attribute] for documentation
#[proc_macro_attribute]
pub fn my_attribute(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Blank attribute that provides #[my_attribute2] for documentation
#[proc_macro_attribute]
pub fn my_attribute2(
    _attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    item
}

/// Duplicate the input tokens, replacing `NAME` with each attribute
///
/// The following creates two structs (`Foo`/`Bar`) with one field (`foo`/`bar`)
///
/// ```
/// #[attribute::dup([Foo, foo], [Bar, bar])]
/// struct NAME_0 {
///     NAME_1: u32
/// }
///
/// let foo = Foo { foo: 0 };
/// let bar = Bar { bar: 0 };
/// ```
#[proc_macro_attribute]
pub fn dup(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    struct Idents(Vec<TokenTree>);

    impl Parse for Idents {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            if input.peek(Ident) {
                let i = input.parse()?;
                Ok(Self(vec![i]))
            } else {
                let i;
                bracketed!(i in input);
                let i: Punctuated<_, Token![,]> = Punctuated::parse_terminated(&i)?;
                Ok(Self(i.into_iter().collect()))
            }
        }
    }

    fn replace_ident(idents: &Idents, ts: TokenStream) -> TokenStream {
        let mut out = TokenStream::new();
        for t in ts {
            match t {
                TokenTree::Group(group) => {
                    Group::new(group.delimiter(), replace_ident(idents, group.stream()))
                        .to_tokens(&mut out);
                }
                TokenTree::Ident(i) => {
                    if idents.0.len() == 1 && i == "NAME" {
                        idents.0[0].to_tokens(&mut out);
                        continue;
                    }

                    let name = i.to_string();
                    if let Some(rest) = name.strip_prefix("NAME_") {
                        let x: usize = rest.parse().unwrap();
                        idents.0[x].to_tokens(&mut out);
                    } else {
                        i.to_tokens(&mut out);
                    }
                }
                TokenTree::Punct(punct) => punct.to_tokens(&mut out),
                TokenTree::Literal(literal) => literal.to_tokens(&mut out),
            }
        }
        out
    }

    let idents: Punctuated<Idents, Token![,]> =
        parse_macro_input!(attr with Punctuated::parse_terminated);

    let mut out = TokenStream::new();
    match parse_macro_input!(item as syn::Item) {
        syn::Item::Mod(mut item_mod) => {
            let mut content = Vec::new();
            let old_content = &item_mod.content.as_ref().unwrap().1;
            for ident in idents {
                for item in old_content {
                    content.push(syn::Item::Verbatim(replace_ident(
                        &ident,
                        item.into_token_stream(),
                    )))
                }
            }
            item_mod.content.as_mut().unwrap().1 = content;
            out.extend(item_mod.into_token_stream());
        }
        item => {
            for ident in idents {
                let ts = replace_ident(&ident, item.to_token_stream());
                out.extend(ts);
            }
        }
    }

    out.into()
}
