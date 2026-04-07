<!-- Readme generated with `gen-readme.sh` -->

# bauer

[![Crates.io](https://img.shields.io/crates/v/bauer.svg)](https://crates.io/crates/bauer)
[![Documentation](https://docs.rs/bauer/badge.svg)](https://docs.rs/bauer/)
[![Dependency status](https://deps.rs/repo/github/funnyboy-roks/bauer/status.svg)](https://deps.rs/repo/github/funnyboy-roks/bauer)

<!-- cargo-rdme start -->

Bauer is a crate for automatically generating Builder-patterns for your structs!

Not sure what kind of builder you want?  Bauer supports a variety of sub-patterns: Owned,
Borrowed, and even Type-State!

## Examples

```rust
#[derive(Builder)]
#[builder(kind = "type-state")]
pub struct Foo {
    required_field: u32,
    #[builder(default)]
    default_field: u32,
    #[builder(into)]
    converting_field: String,
    #[builder(repeat)]
    repeating_field: Vec<u32>,
    #[builder(repeat, repeat_n = 1..=3)]
    limited_repeating_field: Vec<u32>,
}

let foo: Foo = Foo::builder()
    .required_field(42)
    // .default_field(69) // defaults to 0
    .converting_field("hello world") // calls `.into()` to convert from &str -> String
    .repeating_field(420)
    .repeating_field(1337)
    .limited_repeating_field(0) // If not called 1..=3 times, this will fail
    .build();
```

Check out [the repository](https://github.com/funnyboy-roks/bauer/tree/main/bauer/examples) for more
examples!

## Configuration

### Kinds

Bauer supports generating 3 kinds of builders:

#### **Owned** (default) / **Borrowed**

`"owned"` builders are passed around by value and `"borrowed"` builders are passed by mutable
reference.

#### **Type-State**

`"type-state"` builders use the type-state pattern and generate builds that are validated at
compile-time using the type system.

Builder kinds can be switched between trivially using `#[builder(kind = <kind>)]` on the
struct.

### Builder Attributes

All of the attributes that may be applied to the builder are listed below.  These go inside of
a `#[builder(..)]` attribute.  For a more detailed description and examples, check out the
[`Builder`] or click on the attribute.

| Attribute                                    | Description                                                                                                 | Usage                                        |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| [`kind`]                                     | Set the sub-patten to use for this builder                                                                  | `kind = "borrowed"` or `kind = "type-state"` |
| [`const`]                                    | Make this builder work at compile-time -- some limitations are added, but most features continue working    | `const`                                      |
| [`prefix`/`suffix`]                          | Add a prefix/suffix to all field functions created for this builder                                         | `prefix = "set_"` or `suffix = "_field"`     |
| [`visibility`]                               | Change the visibility of the created builder (defaults to the same visibility as the struct)                | `prefix = "set_"` or `suffix = "_field"`     |
| [`crate`]                                    | Override the name of the crate when expanding macros (defaults to `bauer`)                                  | `prefix = "set_"` or `suffix = "_field"`     |
| [`attribute`/`attributes`]                   | Set attribute(s) on the generated builder struct                                                            | `attribute(#[foo])`                          |
| [`doc`/`docs`]                               | Set documentation items on the generated builder struct                                                     | `doc(<doc strings>)`                         |
| [`build_fn`]                                 | Set details about the build function (`attributes`, `doc`, `rename`)                                        | `build_fn(...)`                              |
| [`builder_fn`]                               | Set details about the builder function added to the struct (`attributes`, `doc`, `rename`)                  | `builder_fn(...)`                            |
| [`error`]                                    | Set details about the generated error enum (`attributes`, `doc`, `rename`, `force`)                         | `error(...)`                                 |

[`kind`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#kind
[`const`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#const
[`prefix`/`suffix`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#prefixsuffix
[`visibility`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#visibility
[`crate`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#crate
[`attribute`/`attributes`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#attributes
[`doc`/`docs`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#doc
[`build_fn`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#build_fn
[`builder_fn`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#builder_fn
[`error`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#error

### Field Attributes

All of the attributes that may be applied to fields are listed below.  These go inside of a
`#[builder(..)]` attribute.  For a more detailed description and examples, check out the
[`Builder`] or click on the attribute.

|   Attribute                            | Description                                                                                                 | Usage                              |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| [`skip`]                               | Skip this field in the builder.  No other attributes may be specified when this is used.                    | `skip` or `skip = <value>`         |
| [`default`]                            | Specify a default value or use [`Default`]                                                                  | `default` or `default = <value>`   |
| [`repeat`]                             | Allow repating call to add items to a structure                                                             | `repeat` or `repeat = <type>`      |
| [`repeat_n`]                           | Contorl the number times a `repeat` field is allowed to be set.  This controls the length of the final data | `repeat_n = 1..` or `repeat_n = 4` |
| [`collector`]                          | Use a custom collector for converting into the target data structure (default: [`FromIterator::from_iter`]) | `collector = <function>`           |
| [`into`]                               | Make functions accept `impl `[`Into`]`<Field>`                                                              | `into`                             |
| [`tuple`]                              | Make functions accept tuple items as separate arguments                                                     | `tuple` or `tuple(x, y)`           |
| [`adapter`]                            | Fully cusotmise how functions take arguments and convert them into the field value                          | `adapter = \|<arg>: <ty>\| <expr>` |
| [`rename`]                             | Rename the function that is generated for the field                                                         | `rename = <name>`                  |
| [`skip_prefix`/`skip_suffix`]          | Skip using the prefix/suffix from the builder attribute                                                     | `skip_prefix` or `skip_suffix`     |
| [`attribute`/`attributes`][field_attr] | Set attribute(s) on the function generated for this field                                                   | `attribute(#[foo])`                |
| [`doc`/`docs`][field_doc]              | Set documentation items on the function generated for this field                                            | `doc(<doc strings>)`               |

[`skip`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#skip
[`default`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#default
[`repeat`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#repeat
[`repeat_n`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#repeat_n
[`collector`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#collector
[`into`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#into
[`tuple`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#tuple
[`adapter`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#adapter
[`rename`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#rename
[`skip_prefix`/`skip_suffix`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#skip_prefixskip_suffix
[field_attr]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#attributes-1
[field_doc]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#doc-1

<!-- cargo-rdme end -->

[`Builder`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html
