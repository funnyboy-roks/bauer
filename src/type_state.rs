use std::collections::{HashSet, VecDeque};

use convert_case::{Case, Casing};
use proc_macro2::{Span, TokenStream};
use quote::{ToTokens, format_ident, quote};
use syn::{DeriveInput, Ident};

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

    let mut out = TokenStream::new();

    let fields_pascal: Vec<_> = fields
        .iter()
        .map(|f| Ident::new(&f.ident.to_string().to_case(Case::Pascal), f.ident.span()))
        .collect();

    let field_structures: Vec<_> = fields
        .iter()
        .zip(fields_pascal.iter())
        .map(|(f, name)| {
            if f.wrapped_option {
                todo!("Optional fields")
            } else {
                let set = format_ident!("{}Set", name, span = f.ident.span());
                let unset = format_ident!("{}Unset", name, span = f.ident.span());
                (set, unset)
            }
        })
        .collect();

    for (set, unset) in &field_structures {
        out.extend(quote! {
            #[non_exhaustive]
            struct #set;
            #[non_exhaustive]
            struct #unset;
        });
    }

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    let impl_generics = remove_open_lt(impl_generics);
    let ty_generics = remove_open_lt(ty_generics);

    let field_decls: TokenStream = fields
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if let Some(Repeat { .. }) = &f.attr.repeat {
                // quote! {
                //     #ident: ::std::vec::Vec<#inner_ty>,
                // }
                todo!("Repeat fields");
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

    let (set_fields, unset_fields): (Vec<_>, Vec<_>) =
        field_structures.iter().map(|(a, b)| (a, b)).collect();

    out.extend(quote! {
        struct #builder <#(#fields_pascal,)* #ty_generics> {
            #field_decls
            #phantom
        }

        impl <#(#fields_pascal,)* #impl_generics> #builder <#(#fields_pascal,)* #ty_generics> {
        }

        impl <#impl_generics> #builder<#(#unset_fields,)* #ty_generics> #where_clause {
            pub fn new() -> Self {
                Self {
                    #(#field_names: ::core::default::Default::default(),)*
                    #state: ::core::marker::PhantomData,
                }
            }
        }

        impl <#impl_generics> #builder<#(#set_fields,)* #ty_generics> #where_clause {
            pub fn build(self) -> #ident<#ty_generics> {
                #ident {
                    #(#field_names: self.#field_names.unwrap()),*
                }
            }
        }
    });

    for (i, (f, pascal)) in fields.iter().zip(fields_pascal.iter()).enumerate() {
        let impl_generics_fields = fields_pascal[..i]
            .iter()
            .chain(fields_pascal.iter().skip(i + 1));

        let struct_generics_fields = fields_pascal[..i]
            .iter()
            .chain(std::iter::once(unset_fields[i]))
            .chain(fields_pascal.iter().skip(i + 1));

        let return_struct_generics_fields = fields_pascal[..i]
            .iter()
            .chain(std::iter::once(set_fields[i]))
            .chain(fields_pascal.iter().skip(i + 1));

        let other_field_names = field_names[..i]
            .iter()
            .chain(field_names.iter().skip(i + 1));

        let (args, value) = f.attr.to_args_and_value(&f.ty, &f.ident);
        let fn_ident = f.function_ident(builder_attr);

        let name = &f.ident;

        out.extend(quote! {
            impl <#(#impl_generics_fields,)* #impl_generics> #builder <#(#struct_generics_fields,)* #ty_generics> {
                pub fn #fn_ident(self, #args) -> #builder <#(#return_struct_generics_fields,)* #ty_generics> {
                    #builder {
                        #(#other_field_names: self.#other_field_names,)*
                        #name: Some(#value),
                        #state: ::core::marker::PhantomData,
                    }
                }
            }
        });
    }

    out
}
