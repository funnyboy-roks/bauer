use std::collections::{HashSet, VecDeque};

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{DeriveInput, Ident, spanned::Spanned};

use crate::{BuilderAttr, BuilderField, Repeat};

fn remove_open_lt(tokens: impl ToTokens) -> TokenStream {
    let mut tokens = tokens
        .to_token_stream()
        .into_iter()
        .collect::<VecDeque<_>>();
    if tokens.is_empty() {
        quote! {}
    } else {
        tokens.pop_front();
        tokens.pop_back();
        tokens.into_iter().collect()
    }
}

pub fn type_state_builder(
    builder_attr: &BuilderAttr,
    input: &DeriveInput,
    fields: &[BuilderField],
) -> TokenStream {
    let ident = &input.ident;
    let builder = format_ident!("{}Builder", ident);

    let generic_fields: Vec<_> = fields.iter().filter(|f| f.attr.repeat.is_none()).collect();

    let mut out = TokenStream::new();

    let fields_pascal: Vec<_> = generic_fields
        .iter()
        .map(|f| Ident::new(&f.ident.to_string().to_case(Case::Pascal), f.ident.span()))
        .collect();

    let (set_fields, unset_fields): (Vec<_>, Vec<_>) = generic_fields
        .iter()
        .zip(fields_pascal.iter())
        .map(|(f, name)| {
            let set = format_ident!("{}Set", name, span = f.ident.span());
            let unset = format_ident!("{}Unset", name, span = f.ident.span());
            (set, unset)
        })
        .collect();

    out.extend(set_fields.iter().chain(unset_fields.iter()).map(|f| {
        quote! {
            #[non_exhaustive]
            struct #f;
        }
    }));

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let impl_generics = remove_open_lt(impl_generics);
    let ty_generics = remove_open_lt(ty_generics);

    let field_decls: TokenStream = fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if let Some(Repeat { inner_ty, .. }) = &f.attr.repeat {
                quote! {
                    #ident: ::std::vec::Vec<#inner_ty>,
                }
            } else {
                let ty = &f.ty;
                quote! {
                    #ident: ::core::option::Option<#ty>,
                }
            }
        })
        .collect();

    let field_names: Vec<_> = fields.iter().map(|f| &f.ident).collect();

    let mut state = "_state".to_string();
    let names_set = field_names
        .iter()
        .map(|i| i.to_string())
        .collect::<HashSet<_>>();
    while names_set.contains(&*state) {
        state = format!("_{}", state);
    }
    let state = Ident::new(&state, Span::call_site());

    let phantom = quote! {
        #state: ::core::marker::PhantomData<(#(#fields_pascal,)*)>
    };

    let build_fields = fields.iter().map(|field| {
        let name = &field.ident;

        if let Some(Repeat { inner_ty, len, .. }) = &field.attr.repeat {
            assert!(len.is_none());
            quote_spanned! {
                inner_ty.span() =>
                // using associated function syntax as that gives better error messages
                // (i.e., not "call chain may not have expected associated type"
                #name: ::std::iter::FromIterator::from_iter(self.#name.into_iter())
            }
        } else if field.wrapped_option {
            quote! {
                #name: self.#name
            }
        } else if let Some(default) = &field.attr.default {
            if let Some(default) = default {
                if field.attr.into {
                    quote! {
                        #name: self.#name.unwrap_or_else(|| #default.into())
                    }
                } else {
                    quote! {
                        #name: self.#name.unwrap_or_else(|| #default)
                    }
                }
            } else {
                quote_spanned! {
                    field.ty.span() =>
                    #name: self.#name.unwrap_or_default()
                }
            }
        } else {
            quote! {
                #name: self.#name.unwrap()
            }
        }
    });

    let optional_impl_generics = fields
        .iter()
        .zip(fields_pascal.iter())
        .filter_map(|(f, name)| f.optional().then_some(name));

    let build_generics = fields
        .iter()
        .zip(fields_pascal.iter())
        .zip(set_fields.iter())
        .map(|((f, pascal), set)| if f.optional() { pascal } else { set });

    out.extend(quote! {
        struct #builder <#(#fields_pascal,)* #ty_generics> {
            #field_decls
            #phantom
        }

        impl <#impl_generics> #ident <#ty_generics> {
            pub fn builder() -> #builder<#(#unset_fields,)* #ty_generics> {
                #builder::new()
            }
        }

        impl <#impl_generics> #builder<#(#unset_fields,)* #ty_generics> #where_clause {
            pub fn new() -> Self {
                Self {
                    #(#field_names: ::core::default::Default::default(),)*
                    #state: ::core::marker::PhantomData,
                }
            }
        }

        impl <#(#optional_impl_generics,)* #impl_generics> #builder<#(#build_generics,)* #ty_generics> #where_clause {
            pub fn build(self) -> #ident<#ty_generics> {
                #ident {
                    #(#build_fields),*
                }
            }
        }
    });

    for (i, f) in fields.iter().enumerate() {
        let (args, value) = f.attr.to_args_and_value(f.arg_ty(), &f.ident);
        let fn_ident = f.function_ident(builder_attr);

        let name = &f.ident;

        let fun = if f.attr.repeat.is_some() {
            quote! {
                impl <#(#fields_pascal,)* #impl_generics> #builder <#(#fields_pascal,)* #ty_generics> {
                    pub fn #fn_ident(self, #args) -> #builder <#(#fields_pascal,)* #ty_generics> {
                        let mut this = self; // rather than have `mut self` in the signature
                        this.#name.push(#value);
                        #builder {
                            #(#field_names: this.#field_names,)*
                            #state: ::core::marker::PhantomData,
                        }
                    }
                }
            }
        } else {
            let impl_generics_fields = fields_pascal[..i]
                .iter()
                .chain(fields_pascal.iter().skip(i + 1));

            let struct_generics_fields = fields_pascal[..i]
                .iter()
                .chain(std::iter::once(&unset_fields[i]))
                .chain(fields_pascal.iter().skip(i + 1));

            let return_struct_generics_fields = fields_pascal[..i]
                .iter()
                .chain(std::iter::once(&set_fields[i]))
                .chain(fields_pascal.iter().skip(i + 1));

            quote! {
                impl <#(#impl_generics_fields,)* #impl_generics> #builder <#(#struct_generics_fields,)* #ty_generics> {
                    pub fn #fn_ident(self, #args) -> #builder <#(#return_struct_generics_fields,)* #ty_generics> {
                        let mut this = self; // rather than have `mut self` in the signature
                        this.#name = Some(#value);
                        #builder {
                            #(#field_names: this.#field_names,)*
                            #state: ::core::marker::PhantomData,
                        }
                    }
                }
            }
        };
        out.extend(fun);
    }

    out
}
