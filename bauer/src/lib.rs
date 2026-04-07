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
//! Check out [the repository](https://github.com/funnyboy-roks/bauer/tree/main/bauer/examples) for more
//! examples!
//!
//! # Configuration
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
//! ## Builder Attributes
//!
//! All of the attributes that may be applied to the builder are listed below.  These go inside of
//! a `#[builder(..)]` attribute.  For a more detailed description and examples, check out the
//! [`Builder`] or click on the attribute.
//!
//! | Attribute                                    | Description                                                                                                 | Usage                                        |
//! | -------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
//! | [`kind`]                                     | Set the sub-patten to use for this builder                                                                  | `kind = "borrowed"` or `kind = "type-state"` |
//! | [`const`]                                    | Make this builder work at compile-time -- some limitations are added, but most features continue working    | `const`                                      |
//! | [`prefix`/`suffix`]                          | Add a prefix/suffix to all field functions created for this builder                                         | `prefix = "set_"` or `suffix = "_field"`     |
//! | [`visibility`]                               | Change the visibility of the created builder (defaults to the same visibility as the struct)                | `prefix = "set_"` or `suffix = "_field"`     |
//! | [`crate`]                                    | Override the name of the crate when expanding macros (defaults to `bauer`)                                  | `prefix = "set_"` or `suffix = "_field"`     |
//! | [`attribute`/`attributes`]                   | Set attribute(s) on the generated builder struct                                                            | `attribute(#[foo])`                          |
//! | [`doc`/`docs`]                               | Set documentation items on the generated builder struct                                                     | `doc(<doc strings>)`                         |
//! | [`build_fn`]                                 | Set details about the build function (`attributes`, `doc`, `rename`)                                        | `build_fn(...)`                              |
//! | [`builder_fn`]                               | Set details about the builder function added to the struct (`attributes`, `doc`, `rename`)                  | `builder_fn(...)`                            |
//! | [`error`]                                    | Set details about the generated error enum (`attributes`, `doc`, `rename`, `force`)                         | `error(...)`                                 |
//!
//! [`kind`]: Builder#kind
//! [`const`]: Builder#const
//! [`prefix`/`suffix`]: Builder#prefixsuffix
//! [`visibility`]: Builder#visibility
//! [`crate`]: Builder#crate
//! [`attribute`/`attributes`]: Builder#attributes
//! [`doc`/`docs`]: Builder#doc
//! [`build_fn`]: Builder#build_fn
//! [`builder_fn`]: Builder#builder_fn
//! [`error`]: Builder#error
//!
//! ## Field Attributes
//!
//! All of the attributes that may be applied to fields are listed below.  These go inside of a
//! `#[builder(..)]` attribute.  For a more detailed description and examples, check out the
//! [`Builder`] or click on the attribute.
//!
//! |   Attribute                            | Description                                                                                                 | Usage                              |
//! | -------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------- |
//! | [`skip`]                               | Skip this field in the builder.  No other attributes may be specified when this is used.                    | `skip` or `skip = <value>`         |
//! | [`default`]                            | Specify a default value or use [`Default`]                                                                  | `default` or `default = <value>`   |
//! | [`repeat`]                             | Allow repating call to add items to a structure                                                             | `repeat` or `repeat = <type>`      |
//! | [`repeat_n`]                           | Contorl the number times a `repeat` field is allowed to be set.  This controls the length of the final data | `repeat_n = 1..` or `repeat_n = 4` |
//! | [`collector`]                          | Use a custom collector for converting into the target data structure (default: [`FromIterator::from_iter`]) | `collector = <function>`           |
//! | [`into`]                               | Make functions accept `impl `[`Into`]`<Field>`                                                              | `into`                             |
//! | [`tuple`]                              | Make functions accept tuple items as separate arguments                                                     | `tuple` or `tuple(x, y)`           |
//! | [`adapter`]                            | Fully cusotmise how functions take arguments and convert them into the field value                          | `adapter = \|<arg>: <ty>\| <expr>` |
//! | [`rename`]                             | Rename the function that is generated for the field                                                         | `rename = <name>`                  |
//! | [`skip_prefix`/`skip_suffix`]          | Skip using the prefix/suffix from the builder attribute                                                     | `skip_prefix` or `skip_suffix`     |
//! | [`attribute`/`attributes`][field_attr] | Set attribute(s) on the function generated for this field                                                   | `attribute(#[foo])`                |
//! | [`doc`/`docs`][field_doc]              | Set documentation items on the function generated for this field                                            | `doc(<doc strings>)`               |
//!
//! [`skip`]: Builder#skip
//! [`default`]: Builder#default
//! [`repeat`]: Builder#repeat
//! [`repeat_n`]: Builder#repeat_n
//! [`collector`]: Builder#collector
//! [`into`]: Builder#into
//! [`tuple`]: Builder#tuple
//! [`adapter`]: Builder#adapter
//! [`rename`]: Builder#rename
//! [`skip_prefix`/`skip_suffix`]: Builder#skip_prefixskip_suffix
//! [field_attr]: Builder#attributes-1
//! [field_doc]: Builder#doc-1

/// The main macro
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
/// When a builder can fail, the `.build()` function will return a [`Result`] that contains the
/// built value or an error for the problem.  The current errors are:
///
/// - `Missing{Field}` - A required field is missing
/// - `Range{Field}(usize)` - A field was not specified with the correct number of arguments.  The
///   specified quantity is in the enum.
///
/// Where `{Field}` is replaced with the PascalCase version of the field name.
///
/// ## Type-State Builder
///
/// If the builder kind is `"type-state"`, then all errors will be presented as type-errors at
/// compile-time and the `.build()` function will not return a [`Result`]. (unless
/// [`error`]`(force)` is set).
///
/// ## Forcing Results
///
/// If you wish to force the generated `.build()` function to always return a [`Result`], add the
/// [`error`]`(force)` attribute to the builder.
///
/// # Builder Attributes
///
/// ## **`kind`**
///
/// ### `"owned"` (default)
///
/// All builder functions accept `self` and return `Self`.
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(kind = "owned")]
/// pub struct Foo {
///     a: u32,
/// }
///
/// let foo: Foo = Foo::builder()
///     .a(42)
///     .build()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ### `"borrowed"`
///
/// All builder functions accept `&mut self` and return `&mut Self`.
///
/// This pattern is ideal for builders that need to be dynamic because passing it to functions and
/// using it it loops tends to be more ergonomic.
///
/// _Note: After calling `.build()`, the builder is reset_
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(kind = "borrowed")]
/// pub struct Foo {
///     #[builder(repeat)]
///     values: Vec<u32>,
/// }
///
/// let mut builder = Foo::builder();
/// for x in 0..3 {
///     builder.values(x);
/// }
/// let foo: Foo = builder.build();
/// assert_eq!(foo.values, [0, 1, 2]);
/// ```
///
/// ### `"type-state"`
///
/// The builder and its functions are generated in a way that uses the [type-state pattern].  This
/// means that things like required fields can be enforced at compile-time and will refuse to
/// compile if required fields are not set correctly.
///
/// The `.build()` function will never return an error since erroneous calls will fail to compile.
///
/// This can make error messages harder to decode, but it does provide a static guarantee that the
/// builder was used correctly at compile-time.
///
/// [type-state pattern]: https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html
///
/// ```compile_fail
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(kind = "type-state")]
/// pub struct Foo {
///     a: u32,
/// }
///
/// let foo: Foo = Foo::builder().build(); // fails to compile because `a` is missing
/// ```
///
/// ## **`const`**
///
/// Make the generated builder work at compile-time.
///
/// Using `const` creates some limitations for the builder, primarily:
///
/// - All types need to be constructable at compile-time
/// - [`repeat`] only works on arrays ([`repeat_n`] is disabled)
/// - [`adapter`]s must be const (no syntax change needed, but the body needs to work in const)
/// - [`into`] is disabled
/// - [`default`] requires the default value to be specified and be const (`default = "<expression>"`)
///
/// `const` works best with type-state builders since their `.build()` function can't fail, but it
/// does work with all builders, error handling just takes more thought.
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(kind = "type-state", const)]
/// pub struct Foo {
///     a: u32,
/// }
///
/// const FOO: Foo = Foo::builder()
///     .a(42)
///     .build();
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
///     .build()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## **`visibility`**
///
/// Default: visibility of the struct
///
/// Set the visibility for the generated builder struct
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
/// The name of this crate in the current crate.  This should only need to be changed if you rename
/// the dependency in your `Cargo.toml`
///
/// ```ignore
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// #[builder(crate = not_bauer)]
/// pub struct Foo {
///     a: u32,
/// }
/// ```
///
/// ## **`attributes`**
///
/// Any attributes specified in `attributes` will be added to the generated builder for this field.
/// You may also use `attribute` instead of `attributes`.
///
/// The contents may be wrapped with either `()` or `{}` and attributes may optionally be separated
/// using commas.
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
/// )]
/// pub struct Foo {
///     field: u32,
/// }
/// ```
///
/// ## **`doc`**
///
/// Add documentation to the generated builder struct or the generated `.build()` function
///
/// `#[doc]` attributes may also be added using this attribute, i.e., `doc(hidden)`.
///
/// The contents may be wrapped with either `()` or `{}` and attributes may optionally be separated
/// using commas.
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// #[builder(
///     doc {
///         /// Some documentation for my field
///     },
/// )]
/// pub struct Foo {
///     field: u32,
/// }
/// ```
///
/// ## **`build_fn`**
///
/// Specify details surrounding the generated `.build()` function on the builder.  There are a few
/// attributes that may be specified here:
///
/// - `attributes` - Specify attributes to be applied to the build function (see
///   [`attributes`](#attributes))
/// - `doc` - Add documentation to the generated build function (see [`doc`](#doc))
/// - `rename = <name>` - Rename the build function from the default of `build`
///
/// The contents may be wrapped with either `()` or `{}` and attributes may optionally be separated
/// using commas.
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// #[builder(
///     build_fn {
///         attributes {
///             #[my_attribute]
///             #[my_attribute2]
///         },
///         doc {
///             /// Some documentation about the build function
///         },
///         rename = "finish",
///     },
/// )]
/// pub struct Foo {
///     field: u32,
/// }
///
/// let foo: Foo = Foo::builder()
///     .field(3)
///     .finish()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## **`builder_fn`**
///
/// Specify details surrounding the generated `.builder()` function on the struct.  There are a few
/// attributes that may be specified here:
///
/// - `attributes` - Specify attributes to be applied to the build function (see
///   [`attributes`](#attributes))
/// - `doc` - Add documentation to the generated build function (see [`doc`](#doc))
/// - `rename = <name>` - Rename the build function from the default of `build`
///
/// The contents may be wrapped with either `()` or `{}` and attributes may optionally be separated
/// using commas.
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// #[builder(
///     builder_fn {
///         attributes {
///             #[my_attribute]
///             #[my_attribute2]
///         },
///         doc {
///             /// Some documentation about the builder function
///         },
///         rename = "renamed_function",
///     },
/// )]
/// pub struct Foo {
///     field: u32,
/// }
///
/// let foo: Foo = Foo::renamed_function()
///     .field(3)
///     .build()?;
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## **`error`**
///
/// Specify details surrounding the generated error type for the builder.  There are a few
/// attributes that may be specified here:
///
/// - `attributes` - Specify attributes to be applied to error enum (see
///   [`attributes`](#attributes)) _[This field is ignored on Type-State builders]_
/// - `doc` - Add documentation to the generated error enum (see [`doc`](#doc)) _[This field is ignored on Type-State builders]_
/// - `rename = <name>` - Rename the error enum from the default `{struct}BuildError` _[This field is ignored on Type-State builders]_
/// - `force` - Force the builder to return an error.  This is the error enum for Owned and
///   Borrowed builders, and [`Infallible`] on Type-State.
///
/// The contents may be wrapped with either `()` or `{}` and attributes may optionally be separated
/// using commas.
///
/// [`Infallible`]: std::convert::Infallible
///
/// ```
/// # use bauer_macros::Builder;
/// # use attribute::{my_attribute, my_attribute2};
/// #[derive(Builder)]
/// #[builder(
///     error {
///         attributes {
///             #[my_attribute]
///             #[my_attribute2]
///         },
///         doc {
///             /// Some documentation about the build function
///         },
///         rename = "FooBuildFailure",
///     },
/// )]
/// pub struct Foo {
///     field: u32,
/// }
///
/// let result: Result<Foo, FooBuildFailure> = Foo::builder()
///     .field(3)
///     .build();
/// # let _ = result;
/// ```
///
/// # Fields Attributes
///
/// ## **`skip`**
///
/// Argument: Optional Expression
///
/// Prevent a field from being in the builder.  If provided with no expression, the value will be
/// created from [`Default`].  The expression provided may access the values of all other fields
/// that are not skipped.  These can be accessed like variables using the name of the field.
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(skip)]
///     a: u32,
///     #[builder(skip = *c as f32)] // uses field `c`
///     b: f32,
///     c: u32,
/// }
///
/// let foo = Foo::builder()
///     .c(42)
///     .build()?;
/// assert_eq!(foo.a, 0);
/// assert_eq!(foo.b, 42.0);
/// assert_eq!(foo.c, 42);
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## **`default`**
///
/// Argument: Optional String
///
/// If no default value is provided, the field will attempt to be set using the [`Default`] trait.  
/// Otherwise, the passed string will be parsed as an expression and used to set the default (only
/// run when `.build()` is called and no value has been set)
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(default)]
///     a: u32, // defaults to 0
///     #[builder(default = "std::f32::consts::PI")]
///     b: f32, // defaults to PI
/// }
///
/// let foo = Foo::builder().build();
/// assert_eq!(foo.a, 0);
/// assert_eq!(foo.b, std::f32::consts::PI);
///
/// let foo = Foo::builder()
///     .a(42)
///     .build();
/// assert_eq!(foo.a, 42);
/// assert_eq!(foo.b, std::f32::consts::PI);
/// ```
///
/// ## **`repeat`**
///
/// Make the generated method consume the "inner type" and build the field type at the end.  By
/// default it uses [`FromIterator`] to build the final type, but that may be overridden with the
/// [`collector`] attribute.
///
/// If the field type has a single generic parameter, then that generic will be chosen as the inner
/// type. If the field has a different number of generics, or if the inner type needs to be
/// different, then the type may be set with `repeat = <inner type>`.
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
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
///
/// assert_eq!(foo.items, [0, 1, 2]);
/// assert_eq!(foo.chars, "abc");
/// ```
///
/// ## **`repeat_n`**
///
/// Ensure that the length of items supplied via [`repeat`] is within a certain range.
///
/// The [`repeat`] must be specified before `repeat_n`.
///
/// For Owned and Borrowed builders, the range may be any statement that belongs on the left side
/// of a match statement.  For Type-State builders, the usage is limited to integers (`N`), closed
/// ranges (`N..M` or `N..=M`), and lower-bounded ranges (`N..`).  The length of a range is limited
/// to 64 in order to protect against very slow compile-time.  If a larger range is required, the
/// `unlimited_range` feature may be enabled.
///
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Debug, Builder)]
/// pub struct Foo {
///     #[builder(repeat, repeat_n = 2..=3)]
///     items: Vec<u32>,
/// }
///
/// let foo = Foo::builder().items(0).items(1).items(2).build();
/// assert!(foo.is_ok());
///
/// let foo = Foo::builder().items(0).build().unwrap_err();
/// assert_eq!(foo, FooBuildError::RangeItems(1));
/// ```
///
/// ## **`rename`**
///
/// Change the name of the generated function from the default value matching the field name.
///
/// Note: This still applies the prefix/suffix.  To skip those use [`skip_prefix`]/[`skip_suffix`]
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(repeat, rename = "item")]
///     items: Vec<u32>,
/// }
///
/// let foo = Foo::builder()
///     .item(0)
///     .item(1)
///     .build();
/// assert_eq!(foo.items, [0, 1]);
/// ```
///
/// ## **`skip_prefix`**/**`skip_suffix`**
///
/// If a [`prefix`] or [`suffix`] was set in the builder attributes, skip applying those for this
/// field.  This is especially useful in combination with [`rename`].
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
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
/// assert_eq!(foo.items, [0, 1]);
/// ```
///
/// ## **`into`**
///
/// Make the generated function accept `impl `[`Into`]`<FieldType>`.  This requires the field type
/// to implement [`From`] on whatever value is passed in.
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(into)]
///     a: String,
/// }
///
/// let foo = Foo::builder()
///     .a("hello")
///     .build()?;
/// assert_eq!(foo.a, "hello");
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## **`tuple`**
///
/// Make generated function accept each item of the tuple as a separate parameters to the function.
///
/// By default, the names of the parameters are just `field_0`, `field_1`, etc.  However, if names
/// are specified using `tuple(name1, name2, ...)`, they will be used for the names of the
/// parameters to the function.
///
/// Note: If used with [`repeat`], `tuple` must come after.
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
/// Create a custom implementation for converting from arguments to a value.
///
/// An adapter uses the closure syntax where all arguments have their type specified.  The body of
/// the closure will then be used to generate the value.  Multiple parameters may be used and their
/// names and types will appear in the generated signature.
///
/// Conflicts with [`into`] and [`tuple`].
///
/// ```
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(adapter = |x: u32, y: u32| format!("x={}, y={}", x, y))]
///     point: String,
/// }
///
/// let foo = Foo::builder()
///     .point(5, 23)
///     .build()?;
/// assert_eq!(foo.point, "x=5, y=23");
/// # Ok::<_, Box<dyn std::error::Error>>(())
/// ```
///
/// ## **`collector`**
///
/// On fields that use [`repeat`], a collector may be specified to use in place of the default
/// [`FromIterator`] in order to collect the added values differently.
///
/// The value passed to a collector must be a function with the following signature:
///
/// ```
/// # type RepeatType = u32;
/// # type FieldType = usize;
/// fn my_collector(iter: impl ExactSizeIterator<Item = RepeatType>) -> FieldType
/// # {
/// #     // just a simple implementation to be sure the signature is correct
/// #     iter.len()
/// # }
/// #
/// # #[derive(bauer::Builder)]
/// # struct Foo {
/// #     #[builder(repeat = u32, collector = my_collector)]
/// #     field: usize
/// # }
/// #
/// # let foo = Foo::builder().field(0).field(1).field(2).build();
/// # assert_eq!(foo.field, 3);
/// ```
///
/// Where `RepeatType` is the type determined by the [`repeat`] attribute and `FieldType` is the type
/// of the field.
///
/// _Note: Because [`Iterator`] is a super-trait of [`ExactSizeIterator`], it may be used instead._
///
/// [`FromIterator`]: std::iter::FromIterator
///
/// ```
/// fn sum_collector(iter: impl Iterator<Item = u64>) -> u64 {
///     iter.sum()
/// }
///
/// # use bauer_macros::Builder;
/// #[derive(Builder)]
/// pub struct Foo {
///     #[builder(repeat = u64, collector = sum_collector)]
///     sum: u64,
/// }
///
/// let foo = Foo::builder()
///     .sum(21)
///     .sum(34)
///     .sum(55)
///     .build();
/// assert_eq!(foo.sum, 21 + 34 + 55);
/// ```
///
/// ## **`attributes`**
///
/// Any attributes specified in `attributes` will be added to the generated function for this
/// field.  You may also use `attribute` instead of `attributes`.
///
/// The contents may be wrapped with either `()` or `{}` and attributes may optionally be separated
/// using commas.
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
/// Add documentation to the generated function for this field.
///
/// `#[doc]` attributes may also be added using this attribute, i.e., `doc(hidden)`.
///
/// The contents may be wrapped with either `()` or `{}` and attributes may optionally be separated
/// using commas.
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
///
/// [`prefix`]:       #prefixsuffix
/// [`suffix`]:       #prefixsuffix
/// [`error`]:        #error
///
/// [`collector`]:    #collector
/// [`repeat`]:       #repeat
/// [`repeat_n`]:     #repeat_n
/// [`default`]:      #default
/// [`adapter`]:      #adapter
/// [`into`]:         #into
/// [`rename`]:       #rename
/// [`skip_prefix`]:  #skip_prefixskip_suffix
/// [`skip_suffix`]:  #skip_prefixskip_suffix
pub use bauer_macros::Builder;

#[doc(hidden)]
pub mod __private;

pub mod state {
    use crate::__private;

    /// Represents a type that is equal to the constant value `N`
    ///
    /// This trait should never be user-implemented (hence the `Sealed`).  If you receive an error
    /// about this, you likely have provided the wrong number of repeat arguments to a builder.
    #[deprecated = "This trait should not be implemented by hand"]
    pub trait Eq<const N: usize>: __private::sealed::Sealed {}

    /// Represents a type that is in the range of `LOW..HIGH`
    ///
    /// This trait should never be user-implemented (hence the `Sealed`).  If you receive an error
    /// about this, you likely have provided the wrong number of repeat arguments to a builder.
    #[deprecated = "This trait should not be implemented by hand"]
    pub trait RangeExclusive<const LOW: usize, const HIGH: usize>:
        __private::sealed::Sealed
    {
    }

    /// Represents a type that is in the range of `LOW..=HIGH`
    ///
    /// This trait should never be user-implemented (hence the `Sealed`).  If you receive an error
    /// about this, you likely have provided the wrong number of repeat arguments to a builder.
    #[deprecated = "This trait should not be implemented by hand"]
    pub trait RangeInclusive<const LOW: usize, const HIGH: usize>:
        __private::sealed::Sealed
    {
    }

    /// Represents a type that is in the range of `LOW..`
    ///
    /// This trait should never be user-implemented (hence the `Sealed`).  If you receive an error
    /// about this, you likely have provided the wrong number of repeat arguments to a builder.
    pub trait AtLeast<const LOW: usize>: __private::sealed::Sealed {}
}
