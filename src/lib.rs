//! Bauer is a crate for automatically generating Builder-patterns for your structs!
//!
//! Not sure what kind of builder you want?  Bauer supports a variety of sub-patterns: Owned,
//! Borrowed, and even Type-State!
//!
//! # Examples
//!
//! ```rust
//! # use bauer::Builder;
//! #[derive(Builder)]
//! #[builder(kind = "type-state")]
//! pub struct Foo {
//!     required_field: u32,
//!     #[builder(default)]
//!     default_field: u32,
//!     #[builder(into)]
//!     converting_field: String,
//!     #[builder(repeat)]
//!     repeating_field: Vec<u32>,
//!     #[builder(repeat, repeat_n = 1..=3)]
//!     limited_repeating_field: Vec<u32>,
//! }
//!
//! let foo: Foo = Foo::builder()
//!     .required_field(42)
//!     // .default_field(69) // defaults to 0
//!     .converting_field("hello world") // calls `.into()` to convert from &str -> String
//!     .repeating_field(420)
//!     .repeating_field(1337)
//!     .limited_repeating_field(0) // If not called 1..=3 times, this will fail
//!     .build();
//! ```
//!
//! Check out [the repository](https://github.com/funnyboy-roks/bauer/tree/main/examples) for more
//! examples!
//!
//! # Configuration
//!
//! Builders are very configurable.  A few of the biggest features can be found below.  For a more
//! comprehensive collection of features, look at the [`Builder`] macro.
//!
//! ## Kinds
//!
//! Bauer supports generating 3 kinds of builders:
//!
//! ### **Owned** (default) / **Borrowed**
//!
//! `"owned"` builders are passed around by value and `"borrowed"` builders are passed by mutable
//! reference.
//!
//! ### **Type-State**
//!
//! `"type-state"` builders use the type-state pattern and generate builds that are validated at
//! compile-time using the type system.
//!
//! Builder kinds can be switched between trivially using `#[builder(kind = <kind>)]` on the
//! struct.
//!
//! ## Field Attributes
//!
//! These attributes go in `#[builder(..)]` on individual fields of the structure
//!
//! ### **`default`**
//!
//! Specify a default value for the field to have, or use [`Default::default`]
//!
//! ### **`repeat`**
//!
//! Allow any structure which supports [`FromIterator`] to be specified by calling the function
//! multiple times.  If `repeat_n` is specified, the number of times to repeat is limited.
//!
//! ### **`into`**/**`tuple`**/**`adapter`**
//!
//! Change how the generated builder function handles input.  Can also be used with `repeat`.
//!
//! - `into` will make the function accepet `impl Into<T>`  
//! - `tuple` will make the function accept each item as a separate argument
//! - `adapter` can specify each argument and how they should be converted into the value
//!
//! **There are many more attributes, all can be found on the [`Builder`] macro.**
//!
//! [`Builder`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, format_ident, quote, quote_spanned};
use syn::{DeriveInput, parse::ParseStream, parse_macro_input, spanned::Spanned};

use crate::{
    builder::{BuilderAttr, Kind},
    field::{BuilderField, Repeat},
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
/// use bauer::Builder;
///
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// ## **`prefix`**/**`suffix`**
///
/// Default: `prefix = "", suffix = ""`
///
/// Set the prefix or suffix for the generated builder functions
///
/// ```
/// # use bauer::Builder;
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
/// # use bauer::Builder;
/// #[derive(Builder)]
/// #[builder(visibility = pub(crate))]
/// pub struct Foo {
///     a: u32,
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
/// # use bauer::Builder;
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
    let fields_named: Vec<_> = match data_struct.fields {
        syn::Fields::Named(ref fields_named) => match fields_named
            .named
            .iter()
            .map(|f| BuilderField::parse(f, ident))
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

    let fields: TokenStream2 = fields_named
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

    if attr.kind == Kind::TypeState {
        return type_state::type_state_builder(&attr, &input, &fields_named).into();
    }

    let functions: TokenStream2 = fields_named.iter().map(|f| f.function(&attr)).collect();

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
                len: Some((len, err)),
                ..
            }) = &f.attr.repeat
            {
                variants.push((
                    quote! {
                        #err(usize)
                    },
                    quote!{
                        Self::#err(n) => write!(f, "Invalid number of repeat arguments provided.  Expected {:?}, got {}", #len, n)
                    },
                ));
            }
            variants.into_iter()
        })
        .collect();

    let field_names: Vec<_> = fields_named.iter().map(|f| &f.ident).collect();

    let build_fields = fields_named.iter().map(|field| {
        let name = &field.ident;

        if let Some(Repeat { inner_ty, len, .. }) = &field.attr.repeat {
            if let Some((range, err)) = len {
                quote_spanned! {
                    range.span() =>
                    #name: match self.#name.len() {
                        #range => self.#name.drain(..).collect(),
                        len => return Err(#build_err::#err(len)),
                    }
                }
            } else {
                quote_spanned! {
                    inner_ty.span() =>
                    // using associated function syntax as that gives better error messages
                    // (i.e., not "call chain may not have expected associated type"
                    #name: ::std::iter::FromIterator::from_iter(self.#name.drain(..))
                }
            }
        } else if field.wrapped_option {
            quote! {
                #name: self.#name
            }
        } else if let Some(default) = &field.attr.default {
            if let Some(default) = default {
                if field.attr.into {
                    quote! {
                        #name: self.#name.take().unwrap_or_else(|| #default.into())
                    }
                } else {
                    quote! {
                        #name: self.#name.take().unwrap_or_else(|| #default)
                    }
                }
            } else {
                quote_spanned! {
                    field.ty.span() =>
                    #name: self.#name.take().unwrap_or_default()
                }
            }
        } else {
            let err = field
                .missing_err
                .as_ref()
                .expect("missing_err is set when default is none");
            quote! {
                #name: self.#name.take().ok_or(#build_err::#err)?
            }
        }
    });

    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let build_fn = if build_err_variants.is_empty() {
        quote! {
            #builder_vis fn build(#self_param) -> #ident #ty_generics {
                #ident {
                    #(#build_fields),*
                }
            }
        }
    } else {
        quote! {
            #builder_vis fn build(#self_param) -> ::core::result::Result<#ident #ty_generics, #build_err> {
                Ok(#ident {
                    #(#build_fields),*
                })
            }
        }
    };

    let build_err_enum = if build_err_variants.is_empty() {
        quote! {}
    } else {
        quote! {
            #[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq)]
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

    quote! {
        #build_err_enum

        #builder_vis struct #builder #impl_generics {
            #fields
        }

        impl #impl_generics #builder #ty_generics #where_clause {
            #functions

            #build_fn
        }

        impl #impl_generics ::core::default::Default for #builder #ty_generics #where_clause {
            fn default() -> Self {
                Self {
                    #(#field_names: ::core::default::Default::default()),*
                }
            }
        }

        impl #impl_generics #ident #ty_generics #where_clause {
            #builder_vis fn builder() -> #builder #ty_generics {
                ::core::default::Default::default()
            }
        }
    }
    .into()
}
