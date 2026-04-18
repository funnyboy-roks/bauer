#![doc = include_str!("../README.md")]

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
    DeriveInput, Ident, Pat, parse::ParseStream, parse_macro_input, parse_quote_spanned,
    spanned::Spanned,
};

use crate::{
    attr::builder::{BuilderAttr, Kind},
    attr::field::{BuilderField, Len, Repeat},
    util::parallel_assign,
};

mod attr;
mod type_state;
mod util;

/// A very minimal builder implementation where everything panics
fn failed_builder(
    mut builder_attr: BuilderAttr,
    input: &DeriveInput,
    fields: Vec<BuilderField>,
    errors: &[syn::Error],
) -> TokenStream2 {
    assert!(!errors.is_empty());

    let is_type_state = builder_attr.kind == Kind::TypeState;
    // Some of the simpler functions require that the kind is not type-state and since we're
    // failing, it doesn't really matter.
    builder_attr.kind = Kind::Owned;

    let ident = &input.ident;
    let assert_crate = builder_attr.assert_crate();
    let builder_attributes = &builder_attr.attributes;
    let builder_vis = &builder_attr.vis;
    let builder = format_ident!("{}Builder", ident);
    let build_err = builder_attr.error.name(ident);
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let konst = builder_attr.konst_kw();
    let self_param = builder_attr.self_param();

    let functions: TokenStream2 = fields
        .iter()
        .filter(|f| !f.should_skip())
        .map(|f| f.fail_fn(&builder_attr))
        .collect();

    let (build_err_variants, _) = gen_error_enum(&fields);

    let infallible = (is_type_state || build_err_variants.is_empty()) && !builder_attr.error.force;

    let build_err_enum = if infallible {
        quote! {}
    } else {
        let attributes = &builder_attr.error.attributes;
        quote! {
            #(#attributes)*
            #[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
            #builder_vis enum #build_err {}

            impl ::core::fmt::Display for #build_err {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    panic!("Invalid Builder");
                }
            }

            impl ::core::error::Error for #build_err {}
        }
    };

    let ret_ty = if infallible {
        quote! { #ident #ty_generics }
    } else {
        quote! { ::core::result::Result<#ident #ty_generics, #build_err> }
    };

    let build_fn_attributes = &builder_attr.build_fn.attributes;
    let build_fn_name = &builder_attr.build_fn.name;
    let build_fn = quote! {
        #(#build_fn_attributes)*
        #builder_vis #konst fn #build_fn_name(#self_param) -> #ret_ty {
            panic!("Invalid Builder")
        }
    };

    let into_impl = into_impl(
        &builder_attr,
        input,
        &builder,
        (!infallible).then_some(build_err),
    );

    let builder_fn = builder_fn(input, &builder_attr, &builder);

    let errors = errors.iter().map(syn::Error::to_compile_error);

    quote! {
        #assert_crate

        #build_err_enum

        #(#builder_attributes)*
        #[must_use = "The builder doesn't construct its type until `.build()` is called"]
        #builder_vis struct #builder #impl_generics #where_clause {}

        impl #impl_generics #builder #ty_generics #where_clause {
            #functions

            #build_fn
        }

        impl #impl_generics #builder #ty_generics #where_clause {
            #konst fn new() -> Self {
                panic!("Invalid Builder")
            }
        }

        impl #impl_generics ::core::default::Default for #builder #ty_generics #where_clause {
            fn default() -> Self {
                Self::new()
            }
        }

        #builder_fn

        #into_impl

        #(#errors)*
    }
}

fn into_impl(
    builder_attr: &BuilderAttr,
    input: &DeriveInput,
    builder: &Ident,
    error: Option<impl ToTokens>,
) -> TokenStream2 {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let build_fn_name = &builder_attr.build_fn.name;

    if let Some(build_err) = error {
        quote! {
            #[allow(clippy::infallible_try_from)]
            impl #impl_generics ::core::convert::TryFrom<#builder #ty_generics> for #ident #ty_generics #where_clause {
                type Error = #build_err;

                fn try_from(mut builder: #builder #ty_generics) -> Result<Self, Self::Error> {
                    builder.#build_fn_name()
                }
            }
        }
    } else {
        quote! {
            impl #impl_generics ::core::convert::From<#builder #ty_generics> for #ident #ty_generics #where_clause {
                fn from(mut builder: #builder #ty_generics) -> Self {
                    builder.#build_fn_name()
                }
            }
        }
    }
}

fn builder_fn(input: &DeriveInput, builder_attr: &BuilderAttr, builder: &Ident) -> TokenStream2 {
    let ident = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let konst = builder_attr.konst_kw();
    let builder_vis = &builder_attr.vis;

    let name = &builder_attr.builder_fn.name;
    let attributes = &builder_attr.builder_fn.attributes;

    quote! {
        impl #impl_generics #ident #ty_generics #where_clause {
            #(#attributes)*
            #builder_vis #konst fn #name() -> #builder #ty_generics {
                #builder::new()
            }
        }
    }
}

fn parse_build_attr(input: &DeriveInput, errors: &mut Vec<syn::Error>) -> BuilderAttr {
    let mut out = BuilderAttr::new(input.vis.clone());
    for attr in input.attrs.iter().filter(|a| a.path().is_ident("builder")) {
        if let Err(e) = attr.parse_args_with(|ps: ParseStream| out.parse(ps)) {
            errors.push(e);
        }
    }
    out
}

fn gen_error_enum(fields: &[BuilderField]) -> (Vec<TokenStream2>, Vec<TokenStream2>) {
    fields
        .iter()
        .filter(|f| !f.should_skip())
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
                let error_msg = format!(
                    "Invalid number of repeat arguments provided.  Expected {}, got {{}}",
                    pattern.to_token_stream()
                );
                variants.push((
                    quote! {
                        #error(usize)
                    },
                    quote! {
                        Self::#error(n) => write!(f, #error_msg, n)
                    },
                ));
            }
            variants.into_iter()
        })
        .collect()
}

#[proc_macro_derive(Builder, attributes(builder))]
pub fn builder(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let ident = &input.ident;

    let mut errors = Vec::new();

    let builder_attr: BuilderAttr = parse_build_attr(&input, &mut errors);

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

    let self_param = builder_attr.self_param();
    let builder_vis = &builder_attr.vis;

    let builder = format_ident!("{}Builder", ident);
    let build_err = builder_attr.error.name(ident);
    let inner = format_ident!("__unsafe_builder_content");

    let mut tuple_index = 0;
    let fields = match data_struct.fields {
        syn::Fields::Named(ref fields_named) => {
            //
            fields_named
                .named
                .iter()
                .map(|f| {
                    BuilderField::parse(f, &builder_attr, ident, &mut tuple_index, &mut errors)
                })
                .collect::<Vec<_>>()
        }
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

    let private_module = builder_attr.private_module();

    if !errors.is_empty() {
        return failed_builder(builder_attr, &input, fields, &errors).into();
    }

    if builder_attr.kind == Kind::TypeState {
        return type_state::type_state_builder(&builder_attr, &input, fields).into();
    }

    let (field_types, init): (Vec<_>, Vec<_>) = fields
        .iter()
        .filter(|f| !f.should_skip())
        .map(|f| {
            if let Some(Repeat {
                inner_ty,
                array,
                len,
                ..
            }) = &f.attr.repeat
            {
                if *array {
                    let pattern = match &len {
                        Len::Raw { pattern, .. } => pattern.to_token_stream(),
                        Len::Int { len } => len.to_token_stream(),
                        _ => {
                            unreachable!("If array, then Len::Raw set");
                        }
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

    let functions: TokenStream2 = fields
        .iter()
        .filter(|f| !f.should_skip())
        .map(|f| f.function(&builder_attr, &inner))
        .collect();

    let (build_err_variants, build_err_messages) = gen_error_enum(&fields);

    let not_skipped_field_values = fields.iter().filter(|f| !f.should_skip()).map(|field| {
        let name = &field.ident;
        let wrapped_ty = &field.wrapped_type();
        let field_i = field.tuple_index();

        let value = if let Some(rep @ Repeat { inner_ty, collector, .. }) = &field.attr.repeat {
            if let Len::Raw { pattern, error } = &rep.len {
                let value = if rep.array {
                    quote_spanned! { inner_ty.span()=> {
                        let arr = ::core::mem::replace(&mut inner.#field_i, #private_module::PushableArray::new());
                        arr.into_array()
                            .expect("The match ensures the length of this array is correct")
                    }}
                } else {
                    assert!(!rep.array);
                    assert!(!builder_attr.konst);

                    collector.collect(parse_quote_spanned! {inner_ty.span()=>
                        inner.#field_i.drain(..)
                    })
                };

                if let Pat::Ident(_) = pattern {
                    quote_spanned! { pattern.span()=>
                        if inner.#field_i.len() == #pattern {
                            #value
                        } else {
                            return Err(#build_err::#error(self.#inner.#field_i.len()));
                        }
                    }
                } else {
                    quote_spanned! { pattern.span()=>
                        match inner.#field_i.len() {
                            #pattern => #value,
                            len => return Err(#build_err::#error(len)),
                        }
                    }
                }
            } else {
                assert!(!rep.array);
                assert!(!builder_attr.konst);
                collector.collect(parse_quote_spanned! {inner_ty.span()=>
                    inner.#field_i.drain(..)
                })
            }
        } else if field.wrapped_option {
            quote! { inner.#field_i.take() }
        } else if let Some(default) = &field.attr.default {
            let default = default.to_value(field.attr.into);
            quote! {
                // NOTE: not using Option::unwrap_or_else, since it's not stable in const
                match inner.#field_i.take() {
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
                match inner.#field_i.take() {
                    Some(v) => v,
                    None => return Err(#build_err::#err),
                }
            }
        };

        quote! {{
            let #name: #wrapped_ty = #value;
            #name
        }}
    });

    let not_skipped_fields: Vec<_> = fields
        .iter()
        .filter(|f| !f.should_skip())
        .map(|f| &f.ident)
        .collect();

    let set_not_skipped_fields = parallel_assign(
        not_skipped_fields.iter().copied(),
        not_skipped_field_values,
        quote! {
            let inner = &mut self.#inner;
        },
    );

    let set_skipped_fields = parallel_assign(
        fields.iter().filter(|f| f.should_skip()).map(|f| &f.ident),
        fields.iter().filter_map(BuilderField::skipped_field_value),
        quote! {
            #[allow(unused)]
            let (#(#not_skipped_fields),*) = (#(&#not_skipped_fields),*);
        },
    );

    let finish_fields = fields.iter().map(|field| &field.ident);

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let konst = builder_attr.konst_kw();

    let (ret_ty, ret_val) = if build_err_variants.is_empty() && !builder_attr.error.force {
        (quote! { #ident #ty_generics }, quote! { ret })
    } else {
        (
            quote! { ::core::result::Result<#ident #ty_generics, #build_err> },
            quote! { Ok(ret) },
        )
    };

    let build_fn_attributes = &builder_attr.build_fn.attributes;
    let build_fn_name = &builder_attr.build_fn.name;
    let build_fn = quote! {
        #(#build_fn_attributes)*
        #builder_vis #konst fn #build_fn_name(#self_param) -> #ret_ty {
            #[allow(deprecated)] // #inner is set to deprecated
            let ret = {
                #set_not_skipped_fields
                #set_skipped_fields

                #ident {
                    #(#finish_fields),*
                }
            };
            #ret_val
        }
    };

    let build_err_enum = if build_err_variants.is_empty() && !builder_attr.error.force {
        quote! {}
    } else {
        let attributes = &builder_attr.error.attributes;
        quote! {
            #(#attributes)*
            #[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
            #[allow(enum_variant_names)]
            #builder_vis enum #build_err {
                #(#build_err_variants),*
            }

            impl ::core::fmt::Display for #build_err {
                fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                    use ::core::fmt::Write;
                    match *self {
                        #(#build_err_messages),*
                    }
                }
            }

            impl ::core::error::Error for #build_err {}
        }
    };

    let into_impl = into_impl(
        &builder_attr,
        &input,
        &builder,
        (!build_err_variants.is_empty() || builder_attr.error.force).then_some(build_err),
    );

    let builder_attributes = &builder_attr.attributes;

    let builder_fn = builder_fn(&input, &builder_attr, &builder);

    let assert_crate = builder_attr.assert_crate();
    quote! {
        #assert_crate

        #build_err_enum

        #(#builder_attributes)*
        #[must_use = "The builder doesn't construct its type until `.build()` is called"]
        #builder_vis struct #builder #impl_generics #where_clause {
            #[deprecated = "This field is for internal use only; You almost certainly don't need to touch this. If you encounter a bug or missing feature, file an issue on the repo."]
            #[doc(hidden)]
            #inner: (#(#field_types,)*),
        }

        impl #impl_generics #builder #ty_generics #where_clause {
            #functions

            #build_fn
        }

        impl #impl_generics #builder #ty_generics #where_clause {
            #konst fn new() -> Self {
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

        #builder_fn

        #into_impl
    }
    .into()
}
