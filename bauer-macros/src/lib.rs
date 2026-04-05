use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{DeriveInput, parse::ParseStream, parse_macro_input, spanned::Spanned};

use crate::{
    builder::{BuilderAttr, Kind},
    field::{BuilderField, Len, Repeat},
};

mod builder;
mod field;
mod type_state;
mod util;

/// The main macro.
///
/// # Usage
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(default = "42")]
///     pub field_a: u32,
///     pub field_b: bool,
///     #[builder(into)]
///     pub field_c: String,
///     #[builder(repeat, repeat_n = 1..=3)]
///     pub field_d: Vec<f64>,
/// }
/// ```
///
/// # Errors
///
/// When a builder can fail, the `.build` function will return an `Result` that contains the built
/// value or a descriptive error.
///
/// If any of these cases are true, the `.build` function will return a `Result`:
///
/// **A field is required**  
/// By default all fields are required, barring some exceptions (field is `Option`, field has a
/// default value, field is `repeat`, etc)
///
/// **`repeat_n` is set**
/// If `repeat_n` is set for any field, then `.build` will return an error if the range is not
/// satisfied.
///
/// **Other Cases**
/// There are other cases where `.build` can fail, this list is non-exhaustive.
///
/// ## Type-State Builder
///
/// If `kind` is set to `"type-state"`, then the builder will _not_ return a Result, as all build
/// conditions are validated at compile-time.
///
/// # Builder Attributes
///
/// ## **`kind`**
///
/// ### Possible Values
///
/// **`"owned"`**  
/// The builder functions consume and generate owned values
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(kind = "owned")]
/// pub struct Foo {
///     a: u32,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let foo: Foo = Foo::builder()
///     .a(42)
///     .build()?;
/// # Ok(()) }
/// ```
///
/// **`"borrowed"`**  
/// The builder functions operate on mutable references to the builder
///
/// _Note: After calling `.build()`, the builder is reset_
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(kind = "borrowed")]
/// pub struct Foo {
///     a: u32,
/// }
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let mut builder = Foo::builder();
/// builder.a(42);
/// let foo: Foo = builder.build()?;
/// assert_eq!(foo.a, 42);
/// # Ok(()) }
/// ```
///
/// **`"type-state"`**  
/// The builder and its functions are generated in a way that uses the type-state pattern.  This
/// means that things like required fields can be enforced at compile-time.  Due to the constraints
/// with type-state builders, some attributes may be limited.  All limitations are documented with
/// the attributes.
///
/// The `.build` function will never return an error, as it is only possible to call when building
/// the final structure is infallible.
///
/// ```compile_fail
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(kind = "type-state")]
/// pub struct Foo {
///     a: u32,
/// }
///
/// let foo: Foo = Foo::builder().build(); // fails to compile
/// ```
///
/// Default: `"owned"`
///
/// ## **`const`**
///
/// Make the generated builder work at compile-time.
///
/// Using `const` creates some limitations for the builder, primarily:
///
/// - All types need to be constructable at compile-time
/// - `repeat` only works on arrays (`repeat_n` is disabled)
/// - `adapter`s must be const (no syntax change needed, but the body needs to work in const)
/// - `into` is disabled
/// - `default` requires the default value to be specified (`default = "<expression>"`)
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(crate = not_bauer)]
/// pub struct Foo {
///     a: u32,
/// }
/// ```
///
/// ## **`prefix`**/**`suffix`**
///
/// Default: `prefix = "", suffix = ""`
///
/// Set the prefix or suffix for the generated builder functions
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(prefix = "set_")]
/// pub struct Foo {
///     a: u32,
/// }
///
/// let f = Foo::builder()
///     .set_a(42)
///     .build()
///     .unwrap();
/// ```
///
/// ## **`visibility`**
///
/// Default: visibility of the struct
///
/// Set the visibilty for the created builder
///
/// The visibility can be set to `pub(self)` in order to make the builder private to the current
/// module.
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(visibility = pub(crate))]
/// pub struct Foo {
///     a: u32,
/// }
/// ```
///
/// ## **`crate`**
///
/// Default: `bauer`
///
/// The name of this crate in the current crate.  This should only be needed if you rename the
/// dependency in your `Cargo.toml`
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(crate = not_bauer)]
/// pub struct Foo {
///     a: u32,
/// }
/// ```
///
/// ## **`attributes`** / **`build_fn_attributes`**
///
/// Any attributes specified in `attributes` will be added to the generated builder struct.
/// Similarly, any attributes specified in `build_fn_attributes` will be added to generated
/// `.build()` function.
///
/// You may also use `attribute` instead of `attributes` and `build_fn_attribute` instead of
/// `build_fn_attributes`.
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// #[builder(
///     attributes(
///         #[my_attribute]
///         #[my_attribute2]
///     ),
///     build_fn_attributes(
///         #[my_attribute]
///         #[my_attribute2]
///     ),
/// )]
/// pub struct Foo {
///     field: u32,
/// }
/// ```
///
/// ## **`doc`** / **`build_fn_doc`**
///
/// Add documentation to the generated builder struct or the generated `.build()` function
///
/// `#[doc]` attributes may also be added using this attribute, i.e., `doc(hidden)`.
///
/// Either parentheses `()` or brackets `{}` may be used after `doc`/`build_fn_doc`.
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// #[builder(
///     doc {
///         /// Some documentation for my field
///     },
///     build_fn_doc {
///         /// Some documentation for my field
///     },
/// )]
/// pub struct Foo {
///     field: u32,
/// }
/// ```
///
/// # Fields Attributes
///
/// ## **`default`**
///
/// Argument: Optional String
///
/// If provided, the field does not need to be specified, and will default to the value provided.
/// If not value is provided to the `default` attribute, then [`Default::default`] will be used.
///
/// ```
/// # use bauer_macros::Builder;
/// # const _: &str = stringify!(
/// #[derive(Builder)]
/// # );
/// # #[derive(Builder, PartialEq, Debug)]
/// pub struct Foo {
///     #[builder(default)]
///     a: u32, // defaults to 0
///     #[builder(default = "std::f32::consts::PI")]
///     b: f32, // defaults to PI
/// }
///
/// let foo = Foo::builder().build();
/// assert_eq!(foo, Foo { a: 0, b: std::f32::consts::PI });
///
/// let foo = Foo::builder()
///     .a(42)
///     .build();
/// assert_eq!(foo, Foo { a: 42, b: std::f32::consts::PI });
/// ```
///
/// ## **`repeat`**
///
/// Make the method accept only a single item and build a list from it
///
/// When using a data structure that does not have the inner type as its singular generic, the type
/// can be specified using `repeat = <type>`.
///
/// ```
/// # use bauer_macros::Builder;
/// # const _: &str = stringify!(
/// #[derive(Builder)]
/// # );
/// # #[derive(Builder, PartialEq, Debug)]
/// pub struct Foo {
///     #[builder(repeat)]
///     items: Vec<u32>,
///     #[builder(repeat = char)]
///     chars: String,
/// }
///
/// let foo = Foo::builder()
///     .items(0)
///     .items(1)
///     .items(2)
///     .chars('a')
///     .chars('b')
///     .chars('c')
///     .build();
/// assert_eq!(
///     foo,
///     Foo {
///         items: vec![0, 1, 2],
///         chars: String::from("abc"),
///     },
/// );
/// ```
///
/// ## **`repeat_n`**
///
/// Attribute `repeat` must also be specified.
///
/// Ensure that the length of items supplied via repeat is within a certain range.  The range can
/// be any pattern that may be used in a `match` statement.  If this range is not met, an error
/// will be returned.
///
/// #### Type-state Builder
///
/// When using the type-state kind, the value used is limited to the following (where `N` and `M`
/// are integer literals)
///
/// - Integer Literals (`N`)
/// - Closed Ranges (`N..M` or `N..=M`)
/// - Minimum Ranges (`N..`)
///
/// Note: The length of the range is limited to 64, because big ranges slow compile-time.  If you
/// require a larger range and the compile-time sacrifice is worth it, you can enable the
/// `unlimited_range` feature.
///
/// ```
/// # use bauer_macros::Builder;
/// # const _: &str = stringify!(
/// #[derive(Builder)]
/// # );
/// # #[derive(Builder, PartialEq, Debug)]
/// pub struct Foo {
///     #[builder(repeat, repeat_n = 2..=3)]
///     items: Vec<u32>,
/// }
///
/// let foo = Foo::builder()
///     .items(0)
///     .items(1)
///     .items(2)
///     .build()
///     .unwrap();
/// assert_eq!(foo, Foo { items: vec![0, 1, 2] });
///
/// let foo = Foo::builder()
///     .items(0)
///     .build()
///     .unwrap_err();
/// assert_eq!(foo, FooBuildError::RangeItems(1));
/// ```
///
/// ## **`rename`**
///
/// Make the function that is generated use a different name from field itself.
///
/// ```
/// # use bauer_macros::Builder;
/// # const _: &str = stringify!(
/// #[derive(Builder)]
/// # );
/// # #[derive(Builder, PartialEq, Debug)]
/// pub struct Foo {
///     #[builder(repeat, rename = "item")]
///     items: Vec<u32>,
/// }
///
/// let foo = Foo::builder()
///     .item(0)
///     .item(1)
///     .build();
/// assert_eq!(foo, Foo { items: vec![0, 1] });
/// ```
///
/// ## **`skip_prefix`**/**`skip_suffix`**
///
/// If a prefix or a suffix is specified in the builder attributes, skip applying those to the name
/// of this function.  This is epecially useful with `rename`.
///
/// ```
/// # use bauer_macros::Builder;
/// # const _: &str = stringify!(
/// #[derive(Builder)]
/// # );
/// # #[derive(Builder, PartialEq, Debug)]
/// #[builder(prefix = "set_")]
/// pub struct Foo {
///     #[builder(repeat, rename = "item", skip_prefix)]
///     items: Vec<u32>,
/// }
///
/// let foo = Foo::builder()
///     .item(0)
///     .item(1)
///     .build();
/// assert_eq!(foo, Foo { items: vec![0, 1] });
/// ```
///
/// ## **`into`**
///
/// Make the method accept anything can be turned into the field.
///
/// ```
/// # use bauer_macros::Builder;
/// # const _: &str = stringify!(
/// #[derive(Builder)]
/// # );
/// # #[derive(Builder, PartialEq, Debug)]
/// pub struct Foo {
///     #[builder(into)]
///     a: String,
/// }
///
/// let foo = Foo::builder()
///     .a("hello")
///     .build()
///     .unwrap();
/// assert_eq!(foo, Foo { a: String::from("hello") });
/// ```
///
/// ## **`tuple`**
///
/// Rather than accepting a field that is a tuple by value, accept each element of the tuple as a
/// separate parameters to the setter function.
///
/// If names are specified using `tuple(name1, name2, ...)`, they will be used for the names of the
/// parameters to the function (see example).
///
/// Note: If used with `repeat`, `repeat` must come before `tuple`.
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(tuple)]
///     tuple: (i32, i32),
///     #[builder(tuple(a, b))]
///     tuple_names: (i32, i32),
///     #[builder(into, tuple(a, b))]
///     tuple_into: (String, f64),
///     #[builder(repeat, tuple(foo, bar))]
///     tuples: Vec<(i32, i32)>,
/// }
///
/// let foo = Foo::builder()
///     .tuple(0, 1)
///     .tuple_names(2, 3)
///     .tuple_into("pi", 3.14)
///     .tuples(4, 5)
///     .tuples(6, 7)
///     .build();
/// ```
///
/// ## **`adapter`**
///
/// Create a custom implementation for the generated function.  The adapter uses the closure syntax
/// with types specified and will generate the method accordingly.
///
/// Any number of arguments are allowed and will be used in the generated function.
///
/// Conflicts with `into` and `tuple`.
///
/// ```
/// # use bauer_macros::Builder;
/// # const _: &str = stringify!(
/// #[derive(Builder)]
/// # );
/// # #[derive(Builder, PartialEq, Debug)]
/// pub struct Foo {
///     #[builder(adapter = |x: u32, y: u32| format!("{}/{}", x, y))]
///     field: String,
/// }
///
/// let foo = Foo::builder()
///     .field(5, 23)
///     .build()
///     .unwrap();
/// assert_eq!(foo, Foo { field: String::from("5/23") });
/// ```
///
/// ## **`attributes`**
///
/// Any attributes specified in `attributes` will be added to the generated function for this
/// field.  You may also use `attribute` insted of `attributes`.
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(
///         attributes(
///             #[my_attribute]
///             #[my_attribute2]
///         ),
///     )]
///     field: u32,
/// }
/// ```
///
/// ## **`doc`**
///
/// Add documentation to the field builder.  If documentation is not set using this
/// attribute, then the documentation on the function will be inherited from the field.
///
/// `#[doc]` attributes may also be added using this attribute, i.e., `doc(hidden)`.
///
/// Either parentheses `()` or brackets `{}` may be used after `doc`.
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(
///         doc {
///             /// Some documentation for my field
///         },
///     )]
///     field_a: u32,
///     #[builder(default, doc(hidden))]
///     field_b: u32,
/// }
/// ```
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

    if attr.kind == Kind::TypeState {
        return type_state::type_state_builder(&attr, &input, fields_named).into();
    }

    let private_module = attr.private_module();
    let (fields, init): (Vec<_>, Vec<_>) = fields_named
        .iter()
        .map(|f| {
            if let Some(Repeat {
                inner_ty,
                array,
                len,
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

        if let Some(rep @ Repeat { inner_ty, .. }) = &field.attr.repeat {
            if let Len::Raw { pattern, error } = &rep.len {
                let value = if rep.array {
                    quote_spanned! { inner_ty.span()=> {
                        let arr = ::core::mem::replace(&mut self.#inner.#field_i, #private_module::PushableArray::new());
                        arr.into_array()
                            .expect("The match ensures the length of this array is correct")
                    }}
                } else {
                    quote_spanned! { inner_ty.span()=>
                        self.#inner.#field_i.drain(..).collect()
                    }
                };
                quote_spanned! { pattern.span()=>
                    #name: match self.#inner.#field_i.len() {
                        #pattern => #value, // TODO: Take and then slice.try_into()
                        len => return Err(#build_err::#error(len)),
                    }
                }
            } else {
                quote_spanned! { inner_ty.span()=>
                    // using associated function syntax as that gives better error messages
                    // (i.e., not "call chain may not have expected associated type"
                    #name: ::std::iter::FromIterator::from_iter(self.#inner.#field_i.drain(..))
                }
            }
        } else if field.wrapped_option {
            quote! {
                #name: self.#inner.#field_i.take()
            }
        } else if let Some(default) = &field.attr.default {
            if let Some(default) = default {
                if let Some(span) = field.attr.into {
                    quote_spanned! {span=>
                        #name: self.#inner.#field_i.take().unwrap_or_else(|| #default.into())
                    }
                } else {
                    quote! {
                        // NOTE: not using Option::unwrap_or_else, since it's not stable in const
                        #name: match self.#inner.#field_i.take() {
                            Some(v) => v,
                            None => #default
                        }
                    }
                }
            } else {
                quote_spanned! {
                    field.ty.span() =>
                    #name: self.#inner.#field_i.take().unwrap_or_default()
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

    let build_fn = if build_err_variants.is_empty() {
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
    } else {
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
        quote! {
            impl #impl_generics ::core::convert::From<#builder #ty_generics> for #ident #ty_generics #where_clause {
                fn from(mut builder: #builder #ty_generics) -> Self {
                    builder.build()
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

    quote! {
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
