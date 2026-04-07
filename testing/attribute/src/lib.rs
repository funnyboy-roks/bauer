use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{
    Ident, Token, bracketed, fold::Fold, parse::Parse, parse_macro_input, punctuated::Punctuated,
};

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
    struct Idents(Vec<Ident>);

    impl Parse for Idents {
        fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
            if input.peek(Ident) {
                let i: Ident = input.parse()?;
                Ok(Self(vec![i]))
            } else {
                let i;
                bracketed!(i in input);
                let i = i.parse_terminated(Ident::parse, Token![,])?;
                Ok(Self(i.into_iter().collect()))
            }
        }
    }

    impl Fold for Idents {
        fn fold_ident(&mut self, i: proc_macro2::Ident) -> proc_macro2::Ident {
            if self.0.len() == 1 && i == "NAME" {
                return self.0[0].clone();
            }

            let name = i.to_string();
            if let Some(rest) = name.strip_prefix("NAME_") {
                let x: usize = rest.parse().unwrap();
                self.0[x].clone()
            } else {
                i
            }
        }
    }

    let idents: Punctuated<Idents, Token![,]> =
        parse_macro_input!(attr with Punctuated::parse_terminated);

    let mut out = TokenStream::new();
    match parse_macro_input!(item as syn::Item) {
        syn::Item::Mod(mut item_mod) => {
            let mut content = Vec::new();
            let old_content = &item_mod.content.as_ref().unwrap().1;
            for mut ident in idents {
                for item in old_content {
                    content.push(ident.fold_item(item.clone()))
                }
            }
            item_mod.content.as_mut().unwrap().1 = content;
            out.extend(item_mod.into_token_stream());
        }
        item => {
            for mut ident in idents {
                let ts = ident.fold_item(item.clone());
                out.extend(ts.into_token_stream());
            }
        }
    }

    out.into()
}
