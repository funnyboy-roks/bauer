use proc_macro2::TokenStream;
use quote::{ToTokens, TokenStreamExt};
use syn::{GenericParam, Generics, Token, punctuated::Punctuated};

pub(crate) struct CustomImplGenerics<'a, T> {
    inner: &'a Generics,
    others: Punctuated<T, Token![,]>,
}

impl<'a, T> CustomImplGenerics<'a, T> {
    pub(crate) fn new(inner: &'a Generics, others: impl IntoIterator<Item = T>) -> Self {
        Self {
            inner,
            others: others.into_iter().collect(),
        }
    }
}

// From https://docs.rs/syn/latest/src/syn/generics.rs.html#1199-1249
impl<'a, T> ToTokens for CustomImplGenerics<'a, T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.inner.params.is_empty() && self.others.is_empty() {
            return;
        }

        <Token![<]>::default().to_tokens(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        for param in self.inner.params.pairs() {
            if let GenericParam::Lifetime(def) = param.value() {
                def.lifetime.to_tokens(tokens);
                <Token![,]>::default().to_tokens(tokens);
            }
        }

        for param in self.inner.params.pairs() {
            if let GenericParam::Lifetime(_) = param.value() {
                continue;
            }
            match param.value() {
                GenericParam::Lifetime(_) => unreachable!(),
                GenericParam::Type(param) => {
                    // Leave off the type parameter defaults
                    tokens.append_all(
                        param
                            .attrs
                            .iter()
                            .filter(|a| matches!(a.style, syn::AttrStyle::Outer)),
                    );
                    param.ident.to_tokens(tokens);
                    if !param.bounds.is_empty() {
                        <Token![:]>::default().to_tokens(tokens);
                        param.bounds.to_tokens(tokens);
                    }
                }
                GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    tokens.append_all(
                        param
                            .attrs
                            .iter()
                            .filter(|a| matches!(a.style, syn::AttrStyle::Outer)),
                    );
                    param.const_token.to_tokens(tokens);
                    param.ident.to_tokens(tokens);
                    param.colon_token.to_tokens(tokens);
                    param.ty.to_tokens(tokens);
                }
            }
            <Token![,]>::default().to_tokens(tokens);
        }

        if !self.others.is_empty() {
            self.others.to_tokens(tokens);
        }

        <Token![>]>::default().to_tokens(tokens);
    }
}

pub(crate) struct CustomTypeGenerics<'a, T> {
    inner: &'a Generics,
    others: Punctuated<T, Token![,]>,
}

impl<'a, T> CustomTypeGenerics<'a, T> {
    pub(crate) fn new(inner: &'a Generics, others: impl IntoIterator<Item = T>) -> Self {
        Self {
            inner,
            others: others.into_iter().collect(),
        }
    }
}

impl<'a, T> ToTokens for CustomTypeGenerics<'a, T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.inner.params.is_empty() && self.others.is_empty() {
            return;
        }

        <Token![<]>::default().to_tokens(tokens);

        // Print lifetimes before types and consts, regardless of their
        // order in self.params.
        for param in self.inner.params.pairs() {
            if let GenericParam::Lifetime(def) = *param.value() {
                // Leave off the lifetime bounds and attributes
                def.lifetime.to_tokens(tokens);
                <Token![,]>::default().to_tokens(tokens);
            }
        }

        for param in self.inner.params.pairs() {
            if let GenericParam::Lifetime(_) = **param.value() {
                continue;
            }
            match param.value() {
                GenericParam::Lifetime(_) => unreachable!(),
                GenericParam::Type(param) => {
                    // Leave off the type parameter defaults
                    param.ident.to_tokens(tokens);
                }
                GenericParam::Const(param) => {
                    // Leave off the const parameter defaults
                    param.ident.to_tokens(tokens);
                }
            }
            <Token![,]>::default().to_tokens(tokens);
        }

        if !self.others.is_empty() {
            self.others.to_tokens(tokens);
        }

        <Token![>]>::default().to_tokens(tokens);
    }
}
