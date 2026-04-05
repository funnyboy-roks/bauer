use std::collections::HashSet;

use proc_macro2::TokenTree;
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Ident};

use crate::BuilderField;

pub mod parse;

pub(crate) struct Replace<I: Iterator + Sized> {
    iter: std::iter::Enumerate<I>,
    value: Option<<I as Iterator>::Item>,
    i: usize,
}

impl<I: Iterator> Iterator for Replace<I> {
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let (i, next) = self.iter.next()?;
        if i == self.i {
            Some(self.value.take().expect("i == self.i only occurs once"))
        } else {
            Some(next)
        }
    }
}

pub(crate) trait ReplaceTrait: Iterator + Sized {
    fn replace(self, index: usize, value: Self::Item) -> Replace<Self>;
}

impl<I: Iterator> ReplaceTrait for I {
    fn replace(self, index: usize, value: Self::Item) -> Replace<Self> {
        Replace {
            iter: self.enumerate(),
            value: Some(value),
            i: index,
        }
    }
}

#[derive(Clone, Copy)]
pub struct OptionalToken<T>(pub Option<T>);

impl<T> ToTokens for OptionalToken<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(t) = &self.0 {
            t.to_tokens(tokens)
        }
    }
}

fn is_keyword(ident: &Ident) -> bool {
    // Taken from https://docs.rs/syn/latest/src/syn/token.rs.html#692-746
    const KEYWORDS: &[&str] = &[
        "abstract", "as", "async", "auto", "await", "become", "box", "break", "const", "continue",
        "crate", "default", "do", "dyn", "else", "enum", "extern", "final", "fn", "for", "if",
        "impl", "in", "let", "loop", "macro", "match", "mod", "move", "mut", "override", "priv",
        "pub", "raw", "ref", "return", "Self", "self", "static", "struct", "super", "trait", "try",
        "type", "typeof", "union", "unsafe", "unsized", "use", "virtual", "where", "while",
        "yield",
    ];

    KEYWORDS.iter().any(|kw| ident == kw)
}

pub fn escape_ident(ident: Ident) -> Ident {
    if is_keyword(&ident) {
        format_ident!("r#{}", ident)
    } else {
        ident
    }
}

fn extract_idents(tt: &TokenTree, out: &mut HashSet<String>) {
    match tt {
        TokenTree::Group(group) => {
            for tt in group.stream() {
                extract_idents(&tt, out);
            }
        }
        TokenTree::Ident(ident) => {
            out.insert(ident.to_string());
        }
        TokenTree::Punct(_) => {}
        TokenTree::Literal(_) => {}
    }
}

// kind of hacky, but it works :shrug:
pub fn known_idents(input: &DeriveInput, fields: &[BuilderField]) -> HashSet<String> {
    let mut out = HashSet::new();
    let (impl_gen, ty_gen, where_clause) = input.generics.split_for_impl();
    let mut ts = quote! { #impl_gen, #ty_gen, #where_clause };
    for x in fields {
        x.ty.to_tokens(&mut ts);
    }
    for t in ts {
        extract_idents(&t, &mut out);
    }

    out
}

pub fn ensure_no_conflict(ident: &mut Ident, known: &HashSet<String>) {
    while known.contains(&ident.to_string()) {
        *ident = format_ident!("_{}", *ident);
    }
}
