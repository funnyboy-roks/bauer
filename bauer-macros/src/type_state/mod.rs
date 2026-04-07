use std::collections::HashMap;

use proc_macro2::TokenStream;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{
    DeriveInput, Ident, Token, TraitBound, Type, TypePath, parse_quote, parse_quote_spanned,
    spanned::Spanned,
};

use crate::{
    BuilderAttr, BuilderField, Len, Repeat,
    field::FieldIdents,
    type_state::generics::{CustomImplGenerics, CustomTypeGenerics},
    util::{ReplaceTrait, ensure_no_conflict, known_idents},
};

mod generics;

fn build_fn(
    builder: &Ident,
    builder_attr: &BuilderAttr,
    fields: &[BuilderField],
    generic_fields: &[&BuilderField],
    len_structs: &HashMap<usize, TraitBound>,
    input: &DeriveInput,
    inner: &Ident,
) -> TokenStream {
    let ident = &input.ident;
    let builder_vis = &builder_attr.vis;
    let private_module = builder_attr.private_module();

    let build_fields = fields.iter().map(|field| {
        let name = &field.ident;
        let pascal = &field.idents.pascal;
        let field_i = field.tuple_index();

        if let Some(Repeat {
            inner_ty,
            len: Len::Int { .. },
            array,
            collector,
        }) = &field.attr.repeat
        {
            let value = if *array {
                quote! { array }
            } else {
                assert!(!builder_attr.konst);
                assert!(!*array);
                collector.collect(parse_quote_spanned! { inner_ty.span()=>
                    array.into_iter()
                })
            };

            quote_spanned! {
                inner_ty.span() =>
                #name: {
                    // SAFETY: The build function can only be called once the array has been
                    // filled and is fully initialised.
                    let array = unsafe { inner.#field_i.assume_init() };
                    #value
                }
            }
        } else if let Some(Repeat {
            inner_ty,
            collector,
            array,
            ..
        }) = &field.attr.repeat
        {
            assert!(!builder_attr.konst);
            assert!(!*array);
            let collect = collector.collect(parse_quote_spanned! { inner_ty.span()=> {
                let _: &::std::vec::Vec<_> = &inner.#field_i; // assert that the types are correct
                inner.#field_i.into_iter()
            }});

            quote_spanned! { inner_ty.span()=>
                #name: #collect
            }
        } else if field.wrapped_option {
            quote! {
                #name: inner.#field_i
            }
        } else if let Some(default) = &field.attr.default {
            let default = default.to_value(field.attr.into);

            quote_spanned! {field.ty.span()=>
                // TODO: make this a function once const traits are stable
                #name: if <#pascal as #private_module::state::BuilderState>::SET {
                    // SAFETY: If #pascal::SET is true, then we have already set #field_i
                    unsafe { inner.#field_i.assume_init() }
                } else {
                    #default
                }
            }
        } else {
            quote! {
                // SAFETY: This function is only accessible if all required fields are set.  This
                // is enusred by the type bounds.
                #name: unsafe { inner.#field_i.assume_init() }
            }
        }
    });

    let build_impl_generics = generic_fields.iter().enumerate().filter_map(|(i, f)| {
        let pascal = &f.idents.pascal;
        if f.optional() {
            Some(quote! {
                #pascal: #private_module::state::BuilderState
            })
        } else if f.optional() || len_structs.contains_key(&i) {
            Some(quote! { #pascal })
        } else {
            None
        }
    });

    let build_generics = generic_fields.iter().enumerate().map(|(i, f)| {
        let FieldIdents {
            count, pascal, set, ..
        } = &f.idents;
        let ty: Type = if len_structs.contains_key(&i) {
            parse_quote! { #count<#pascal> }
        } else if f.optional() {
            parse_quote! { #pascal }
        } else {
            parse_quote! { #set<true> }
        };
        ty
    });

    let impl_generics = CustomImplGenerics::new(&input.generics, build_impl_generics);
    let ty_generics = CustomTypeGenerics::new(&input.generics, build_generics);

    let (_, _, where_clause) = input.generics.split_for_impl();

    let mut builder_where = where_clause.to_token_stream();
    if let Some(where_clause) = where_clause {
        if !where_clause.predicates.trailing_punct() {
            <Token![,]>::default().to_tokens(&mut builder_where);
        }
    } else {
        <Token![where]>::default().to_tokens(&mut builder_where);
    }

    for (&i, len) in len_structs {
        let FieldIdents { pascal, count, .. } = &generic_fields[i].idents;
        builder_where.extend(quote! {
            #count<#pascal>: #len,
        });
    }

    let (_, default_ty_generics, _) = input.generics.split_for_impl();

    let konst = builder_attr.konst_kw();
    let build_fn_attributes = &builder_attr.build_fn_attributes;

    let (build_return, build_return_value, from) = if builder_attr.force_result {
        (
            quote! { ::core::result::Result<#ident #default_ty_generics, ::core::convert::Infallible> },
            quote! { Ok(val) },
            quote! {
                let Ok(built) = builder.build();
                built
            },
        )
    } else {
        (
            quote! { #ident #default_ty_generics },
            quote! { val },
            quote! { builder.build() },
        )
    };

    quote! {
        impl #impl_generics #builder #ty_generics #builder_where {
            #(#build_fn_attributes)*
            #builder_vis #konst fn build(self) -> #build_return {
                #[allow(deprecated)] // #inner is set to deprecated
                {
                    let inner = self.#inner;
                    let val = #ident {
                        #(#build_fields),*
                    };
                    #build_return_value
                }
            }
        }

        impl #impl_generics ::core::convert::From<#builder #ty_generics> for #ident #default_ty_generics #builder_where {
            fn from(builder: #builder #ty_generics) -> Self  {
                #from
            }
        }
    }
}

pub fn type_state_builder(
    builder_attr: &BuilderAttr,
    input: &DeriveInput,
    mut fields: Vec<BuilderField>,
) -> TokenStream {
    let ident = &input.ident;
    let builder = format_ident!("{}Builder", ident);
    let builder_vis = &builder_attr.vis;

    let known_idents = known_idents(input, &fields);
    for f in &mut fields {
        ensure_no_conflict(&mut f.idents.pascal, &known_idents);
        ensure_no_conflict(&mut f.idents.set, &known_idents);
        ensure_no_conflict(&mut f.idents.count, &known_idents);
    }

    let generic_fields: Vec<_> = fields
        .iter()
        .filter(|f| f.attr.repeat.as_ref().is_none_or(|r| r.len.is_some()))
        .collect();

    let mut out = TokenStream::new();

    builder_attr.assert_crate().to_tokens(&mut out);

    let private_module = builder_attr.private_module();
    out.extend(generic_fields.iter().map(|&f| {
        let FieldIdents { count, set, .. } = &f.idents;
        if f.attr.repeat.as_ref().is_some_and(|r| r.len.is_some()) {
            quote! {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #builder_vis struct #count<T>(T); // never constructed, so doesn't really need to be PhantomData
            }
        } else {
            quote! {
                #[doc(hidden)]
                #[allow(non_camel_case_types)]
                #[non_exhaustive]
                #builder_vis struct #set<const SET: bool>;
                impl<const SET: bool> #private_module::sealed::Sealed for #set<SET> {}
                impl<const SET: bool> #private_module::state::BuilderState for #set<SET> {
                    const SET: bool = SET;
                }
            }
        }
    }));

    let mut len_structs = HashMap::new();

    for (i, &f) in generic_fields.iter().enumerate() {
        let Some(repeat) = &f.attr.repeat else {
            continue;
        };

        if repeat.len.is_none() {
            continue;
        }

        let ident = match repeat
            .len
            .to_trait(&builder_attr.krate, &f.idents.count, &mut out)
        {
            Ok(i) => i,
            Err(e) => return e.to_compile_error(),
        };
        len_structs.insert(i, ident);
    }

    let (default_impl_generics, default_ty_generics, where_clause) =
        input.generics.split_for_impl();

    let field_decls = fields.iter().map(|f| {
        let ty = &f.ty;
        if let Some(Repeat {
            inner_ty,
            len: Len::Int { len },
            ..
        }) = &f.attr.repeat
        {
            quote! { ::core::mem::MaybeUninit<[#inner_ty; #len]> }
        } else if let Some(Repeat { inner_ty, .. }) = &f.attr.repeat {
            quote! { ::std::vec::Vec<#inner_ty> }
        } else if f.wrapped_option {
            quote! { ::core::option::Option<#ty> }
        } else {
            quote! { ::core::mem::MaybeUninit<#ty> }
        }
    });

    let init = fields.iter().map(|f| {
        if let Some(Repeat {
            len: Len::Int { .. },
            ..
        }) = &f.attr.repeat
        {
            quote! { ::core::mem::MaybeUninit::uninit() }
        } else if let Some(Repeat { .. }) = &f.attr.repeat {
            quote! { ::std::vec::Vec::new() }
        } else if f.wrapped_option {
            quote! { ::core::option::Option::None }
        } else {
            quote! { ::core::mem::MaybeUninit::uninit() }
        }
    });

    let inner = format_ident!("__unsafe_builder_content");
    let state = format_ident!("__unsafe_builder_state");

    let phantom_generics = generic_fields.iter().map(|f| &f.idents.pascal);
    let phantom = quote! {
        #state: ::core::marker::PhantomData<(#(#phantom_generics,)*)>
    };

    let new_generics = generic_fields.iter().map(|f| {
        let FieldIdents { count, set, .. } = &f.idents;
        if f.attr.repeat.as_ref().is_some_and(|f| f.len.is_some()) {
            quote! { #count<()> }
        } else {
            quote! { #set<false> }
        }
    });

    let struct_generics = CustomImplGenerics::new(
        &input.generics,
        generic_fields.iter().map(|f| &f.idents.pascal),
    );
    let new_generics = CustomTypeGenerics::new(&input.generics, new_generics);

    let konst = builder_attr.konst_kw();
    let attributes = &builder_attr.attributes;

    out.extend(quote! {
        #(#attributes)*
        #[allow(clippy::type_complexity)]
        #[must_use = "The builder doesn't construct its type until `.build()` is called"]
        #builder_vis struct #builder #struct_generics #where_clause {
            #[deprecated = "This field is for internal use only; you almost certainly don't need to touch this. If you encounter a bug or missing feature, file an issue on the repo."]
            #[doc(hidden)]
            #inner: (#(#field_decls,)*),
            #[deprecated = "This field is for internal use only; you almost certainly don't need to touch this. If you encounter a bug or missing feature, file an issue on the repo."]
            #[doc(hidden)]
            #phantom
        }

        impl #default_impl_generics #ident #default_ty_generics #where_clause {
            #builder_vis #konst fn builder() -> #builder #new_generics {
                #builder::new()
            }
        }

        impl #default_impl_generics #builder #new_generics #where_clause {
            #builder_vis #konst fn new() -> Self {
                Self {
                    #inner: (#(#init,)*),
                    #state: ::core::marker::PhantomData,
                }
            }
        }
    });

    // add `fn build()`
    out.extend(build_fn(
        &builder,
        builder_attr,
        &fields,
        &generic_fields,
        &len_structs,
        input,
        &inner,
    ));

    let mut i = 0;
    for f in &fields {
        let (args, value) = f.attr.to_args_and_value(f.arg_ty(), &f.ident);
        let fn_ident = f.function_ident(builder_attr);

        fn ident_to_type(ident: &Ident) -> Type {
            TypePath {
                qself: None,
                path: ident.clone().into(),
            }
            .into()
        }

        let fn_attributes = &f.attr.attributes;

        let field_i = f.tuple_index();
        let value_ty = &f.arg_ty();
        let fun = match &f.attr.repeat {
            Some(Repeat { len: Len::None, .. }) => {
                let impl_generics = CustomImplGenerics::new(
                    &input.generics,
                    generic_fields.iter().map(|f| &f.idents.pascal),
                );
                let ty_generics = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields.iter().map(|f| &f.idents.pascal),
                );
                quote_spanned! {
                    fn_ident.span() =>
                    impl #impl_generics #builder #ty_generics #where_clause {
                        #(#fn_attributes)*
                        #[allow(clippy::type_complexity)]
                        pub #konst fn #fn_ident(self, #args) -> #builder #ty_generics {
                            let value: #value_ty = #value;
                            let mut this = self; // rather than have `mut self` in the signature
                            #[allow(deprecated)] // #inner is set to deprecated
                            {
                                this.#inner.#field_i.push(value);
                                #builder {
                                    #inner: this.#inner,
                                    #state: ::core::marker::PhantomData,
                                }
                            }
                        }
                    }
                }
            }
            Some(Repeat { len, .. }) => {
                let FieldIdents { count, pascal, .. } = &generic_fields[i].idents;

                let impl_generics = CustomImplGenerics::new(
                    &input.generics,
                    generic_fields.iter().map(|f| &f.idents.pascal),
                );
                let ty_generics = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| ident_to_type(&f.idents.pascal))
                        .replace(i, parse_quote! { #count<#pascal> }),
                );

                let ret_ty_generics = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| ident_to_type(&f.idents.pascal))
                        .replace(i, parse_quote! { #count<(#pascal, ())> }),
                );

                let mut field_where = where_clause.to_token_stream();
                if let Some(where_clause) = where_clause {
                    if !where_clause.predicates.trailing_punct() {
                        <Token![,]>::default().to_tokens(&mut field_where);
                    }
                } else {
                    <Token![where]>::default().to_tokens(&mut field_where);
                }

                let add = if let Len::Int { .. } = len {
                    field_where.extend(quote! {
                        #private_module::state::Count::<#pascal>: #private_module::state::Countable,
                    });
                    quote! {
                        let ptr = this.#inner.#field_i.as_mut_ptr();
                        // SAFETY: ptr points to a valid location created by the MaybeUninit
                        unsafe {
                            let ptr: *mut _ = &raw mut (*ptr)[<#private_module::state::Count::<#pascal> as #private_module::state::Countable>::COUNT];
                            ptr.write(value);
                        }
                    }
                } else {
                    quote! { this.#inner.#field_i.push(value); }
                };

                quote_spanned! {
                    fn_ident.span() =>
                    impl #impl_generics #builder #ty_generics #field_where {
                        #(#fn_attributes)*
                        #[allow(clippy::type_complexity)]
                        pub #konst fn #fn_ident(self, #args) -> #builder #ret_ty_generics {
                            let value: #value_ty = #value;
                            let mut this = self; // rather than have `mut self` in the signature
                            #[allow(deprecated)] // #inner is set to deprecated
                            {
                                #add
                                #builder {
                                    #inner: this.#inner,
                                    #state: ::core::marker::PhantomData,
                                }
                            }
                        }
                    }
                }
            }
            None => {
                let impl_generics_fields = CustomImplGenerics::new(
                    &input.generics,
                    generic_fields[..i]
                        .iter()
                        .chain(generic_fields[i + 1..].iter())
                        .map(|f| &f.idents.pascal),
                );

                let FieldIdents { set, .. } = &generic_fields[i].idents;
                let struct_generics_fields = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| ident_to_type(&f.idents.pascal))
                        .replace(i, parse_quote! { #set<false> }),
                );

                let return_struct_generics_fields = CustomTypeGenerics::new(
                    &input.generics,
                    generic_fields
                        .iter()
                        .map(|f| ident_to_type(&f.idents.pascal))
                        .replace(i, parse_quote! { #set<true> }),
                );

                let setter = if f.wrapped_option {
                    quote! {
                        this.#inner.#field_i = Some(value);
                    }
                } else {
                    quote! {
                        this.#inner.#field_i.write(value);
                    }
                };

                quote_spanned! {
                    fn_ident.span() =>
                    impl #impl_generics_fields #builder #struct_generics_fields #where_clause {
                        #(#fn_attributes)*
                        #[allow(clippy::type_complexity)]
                        pub #konst fn #fn_ident(self, #args) -> #builder #return_struct_generics_fields {
                            let value: #value_ty = #value;
                            let mut this = self; // rather than have `mut self` in the signature
                            #[allow(deprecated)] // #inner is set to deprecated
                            {
                                #setter
                                #builder {
                                    #inner: this.#inner,
                                    #state: ::core::marker::PhantomData,
                                }
                            }
                        }
                    }
                }
            }
        };

        out.extend(fun);
        if f.attr.repeat.as_ref().is_none_or(|r| r.len.is_some()) {
            i += 1;
        }
    }

    out
}
