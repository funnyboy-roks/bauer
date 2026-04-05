use proc_macro2::Span;
use syn::{
    AttrStyle, Ident, Meta, Token, bracketed, parse::ParseStream, parse_quote_spanned,
    spanned::Spanned,
};

// from https://docs.rs/syn/latest/src/syn/attr.rs.html#681-689
fn single_parse_outer(input: ParseStream) -> syn::Result<syn::Attribute> {
    let content;
    Ok(syn::Attribute {
        pound_token: input.parse()?,
        style: AttrStyle::Outer,
        bracket_token: bracketed!(content in input),
        meta: content.parse()?,
    })
}

/// Parse a list of (optionally comma separated) attributes
pub fn parse_attributes(input: ParseStream, into: &mut Vec<syn::Attribute>) -> syn::Result<()> {
    // NOTE: can't use Punctuated here as there doesn't seem to be any way to
    // construct one with an optional separator
    while !input.is_empty() {
        let attr = single_parse_outer(input)?;
        into.push(attr);

        if input.is_empty() {
            break;
        }

        let la = input.lookahead1();
        if la.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
        } else if la.peek(Token![#]) {
            continue;
        } else {
            return Err(la.error());
        }
    }
    Ok(())
}

/// Parse a list of (optionally comma separated) doc fields
pub fn parse_docs(
    input: ParseStream,
    doc_span: Span,
    into: &mut Vec<syn::Attribute>,
) -> syn::Result<()> {
    while !input.is_empty() {
        let la = input.lookahead1();

        let attr = if la.peek(Ident) {
            let meta: Meta = input.parse()?;
            parse_quote_spanned! {doc_span=>
                #[doc(#meta)]
            }
        } else if la.peek(Token![#]) {
            let attr = single_parse_outer(input)?;
            if !attr.meta.path().is_ident("doc") {
                return Err(syn::Error::new(
                    attr.meta.span(),
                    "Only `doc` attributes are supported here",
                ));
            }
            attr
        } else {
            return Err(la.error());
        };

        into.push(attr);

        if input.peek(Token![,]) {
            let _: Token![,] = input.parse()?;
        }
    }
    Ok(())
}
