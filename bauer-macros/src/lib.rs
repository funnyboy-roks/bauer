#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
    DeriveInput, TypePath, parse::ParseStream, parse_macro_input, parse_quote, parse_quote_spanned,
    spanned::Spanned,
};

use crate::{
    builder::{BuilderAttr, Kind},
    field::{BuilderField, Len, Repeat},
};

mod builder;
mod field;
mod type_state;
mod util;

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;
    let vis = &input.vis;

    let attr = input.attrs.iter().find(|a| a.path().is_ident("builder"));
    let attr: BuilderAttr = if let Some(attr) = attr {
        match attr.parse_args_with(|ps: ParseStream| BuilderAttr::parse(ps, vis.clone())) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error().into(),
        }
    } else {
        BuilderAttr::new(vis.clone())
    };

    let data_struct = match input.data {
        syn::Data::Struct(ref data_struct) => data_struct,
        syn::Data::Enum(data_enum) => {
            return syn::Error::new(data_enum.enum_token.span(), "Enums are not supported.")
                .to_compile_error()
                .into();
        }
        syn::Data::Union(data_union) => {
            return syn::Error::new(data_union.union_token.span(), "Unions are not supported.")
                .to_compile_error()
                .into();
        }
    };

    let self_param = attr.self_param();
    let builder_vis = &attr.vis;

    let builder = format_ident!("{}Builder", ident);
    let build_err = format_ident!("{}BuildError", ident);
    let inner = format_ident!("__unsafe_builder_content");

    let fields_named: Vec<_> = match data_struct.fields {
        syn::Fields::Named(ref fields_named) => match fields_named
            .named
            .iter()
            .enumerate()
            .map(|(index, f)| BuilderField::parse(f, &attr, ident, index))
            .collect::<Result<_, _>>()
        {
            Ok(v) => v,
            Err(e) => return e.to_compile_error().into(),
        },
        syn::Fields::Unnamed(_) => {
            return syn::Error::new(ident.span(), "Unnamed fields are not supported.")
                .to_compile_error()
                .into();
        }
        syn::Fields::Unit => {
            return syn::Error::new(ident.span(), "Unit structs are not supported.")
                .to_compile_error()
                .into();
        }
    };

    let private_module = attr.private_module();

    if attr.kind == Kind::TypeState {
        return type_state::type_state_builder(&attr, &input, fields_named).into();
    }

    let (fields, init): (Vec<_>, Vec<_>) = fields_named
        .iter()
        .map(|f| {
            if let Some(Repeat {
                inner_ty,
                array,
                len,
                ..
            }) = &f.attr.repeat
            {
                if *array {
                    let Len::Raw { pattern, .. } = &len else {
                        unreachable!("If array, then Len::Raw set");
                    };
                    (
                        quote! { #private_module::PushableArray<#pattern, #inner_ty> },
                        quote! { #private_module::PushableArray::new() },
                    )
                } else {
                    (
                        quote! { ::std::vec::Vec<#inner_ty> },
                        quote! { ::std::vec::Vec::new() },
                    )
                }
            } else {
                let ty = &f.ty;
                (
                    quote! { ::core::option::Option<#ty> },
                    quote! { ::core::option::Option::None },
                )
            }
        })
        .collect();

    let functions: TokenStream2 = fields_named
        .iter()
        .map(|f| f.function(&attr, &inner))
        .collect();

    let (build_err_variants, build_err_messages): (Vec<_>, Vec<_>) = fields_named
        .iter()
        .flat_map(|f| {
            let mut variants = Vec::new();
            if let Some(err) = &f.missing_err {
                let msg = format!("Missing required field '{}'", f.ident);
                variants.push((
                    err.to_token_stream(),
                    quote! { Self::#err => write!(f, #msg) },
                ));
            }
            if let Some(Repeat {
                len: Len::Raw { pattern, error },
                ..
            }) = &f.attr.repeat
            {
                variants.push((
                    quote! {
                        #error(usize)
                    },
                    quote!{
                        Self::#error(n) => write!(f, "Invalid number of repeat arguments provided.  Expected {:?}, got {}", #pattern, n)
                    },
                ));
            }
            variants.into_iter()
        })
        .collect();

    let build_fields = fields_named.iter().map(|field| {
        let name = &field.ident;
        let field_i = field.tuple_index();

        if let Some(rep @ Repeat { inner_ty, collector, .. }) = &field.attr.repeat {
            if let Len::Raw { pattern, error } = &rep.len {
                let value = if rep.array {
                    quote_spanned! { inner_ty.span()=> {
                        let arr = ::core::mem::replace(&mut self.#inner.#field_i, #private_module::PushableArray::new());
                        arr.into_array()
                            .expect("The match ensures the length of this array is correct")
                    }}
                } else {
                    assert!(!rep.array);
                    assert!(!attr.konst);

                    collector.collect(parse_quote_spanned! {inner_ty.span()=>
                        self.#inner.#field_i.drain(..)
                    })
                };

                quote_spanned! { pattern.span()=>
                    #name: match self.#inner.#field_i.len() {
                        #pattern => #value,
                        len => return Err(#build_err::#error(len)),
                    }
                }
            } else {
                assert!(!rep.array);
                assert!(!attr.konst);
                let value = collector.collect(parse_quote_spanned! {inner_ty.span()=>
                    self.#inner.#field_i.drain(..)
                });

                quote_spanned! { inner_ty.span()=>
                    #name: #value
                }
            }
        } else if field.wrapped_option {
            quote! {
                #name: self.#inner.#field_i.take()
            }
        } else if let Some(default) = &field.attr.default {
            let default = default.to_value(field.attr.into);
            quote! {
                // NOTE: not using Option::unwrap_or_else, since it's not stable in const
                #name: match self.#inner.#field_i.take() {
                    Some(v) => v,
                    None => #default
                }
            }
        } else {
            let err = field
                .missing_err
                .as_ref()
                .expect("missing_err is set when default is none");
            quote! {
                // NOTE: not using Option::ok_or, since it's not stable in const
                #name: match self.#inner.#field_i.take() {
                    Some(v) => v,
                    None => return Err(#build_err::#err),
                }
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let konst = attr.konst_kw();
    let builder_fn_attributes = &attr.build_fn_attributes;

    let build_err: TypePath = if build_err_variants.is_empty() && attr.force_result {
        parse_quote! { ::core::convert::Infallible }
    } else {
        parse_quote! { #build_err }
    };

    let build_fn = if !build_err_variants.is_empty() || attr.force_result {
        quote! {
            #(#builder_fn_attributes)*
            #builder_vis #konst fn build(#self_param) -> ::core::result::Result<#ident #ty_generics, #build_err> {
                #[allow(deprecated)] // #inner is set to deprecated
                {
                    Ok(#ident {
                        #(#build_fields),*
                    })
                }
            }
        }
    } else {
        quote! {
            #(#builder_fn_attributes)*
            #builder_vis #konst fn build(#self_param) -> #ident #ty_generics {
                #[allow(deprecated)] // #inner is set to deprecated
                {
                    #ident {
                        #(#build_fields),*
                    }
                }
            }
        }
    };

    let build_err_enum = if build_err_variants.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
            #[allow(enum_variant_names)]
            #builder_vis enum #build_err {
                #(#build_err_variants),*
            }

            impl ::core::fmt::Display for #build_err {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    use ::core::fmt::Write;
                    match self {
                        #(#build_err_messages),*
                    }
                }
            }

            impl ::core::error::Error for #build_err {}
        }
    };

    let into_impl = if build_err_variants.is_empty() {
        let value = if attr.force_result {
            quote! {
                let Ok(built) = builder.build();
                built
            }
        } else {
            quote! { builder.build() }
        };

        quote! {
            impl #impl_generics ::core::convert::From<#builder #ty_generics> for #ident #ty_generics #where_clause {
                fn from(mut builder: #builder #ty_generics) -> Self {
                    #value
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ::core::convert::TryFrom<#builder #ty_generics> for #ident #ty_generics #where_clause {
                type Error = #build_err;

                fn try_from(mut builder: #builder #ty_generics) -> Result<Self, Self::Error> {
                    builder.build()
                }
            }
        }
    };

    let builder_attributes = &attr.attributes;

    let assert_crate = attr.assert_crate();
    quote! {
        #assert_crate

        #build_err_enum

        #(#builder_attributes)*
        #[must_use = "The builder doesn't construct its type until `.build()` is called"]
        #builder_vis struct #builder #impl_generics #where_clause {
            #[deprecated = "This field is for internal use only; You almost certainly don't need to touch this. If you encounter a bug or missing feature, file an issue on the repo."]
            #[doc(hidden)]
            #inner: (#(#fields,)*),
        }

        impl #impl_generics #builder #ty_generics #where_clause {
            #functions

            #build_fn
        }

        impl #impl_generics #builder #ty_generics #where_clause {
            pub #konst fn new() -> Self {
                Self {
                    #inner: (#(#init,)*),
                }
            }
        }

        impl #impl_generics ::core::default::Default for #builder #ty_generics #where_clause {
            fn default() -> Self {
                Self::new()
            }
        }

        impl #impl_generics #ident #ty_generics #where_clause {
            #builder_vis #konst fn builder() -> #builder #ty_generics {
                #builder::new()
            }
        }

        #into_impl
    }
    .into()
}
