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

All of the attributes that may be applied to the builder are listed below.  These go inside of a
`#[builder(..)]` attribute on the struct itself.  For a more detailed description and examples,
check out the [`Builder`] or click on the attribute.

#### Builder Configuration

Attributes that affect the generated struct and other items

| Attribute                                    | Description                                                                                                 | Usage                                        |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| [`kind`]                                     | Set the sub-pattern to use for this builder                                                                 | `kind = "borrowed"` or `kind = "type-state"` |
| [`const`]                                    | Make this builder work at compile-time -- some limitations are added, but most features continue working    | `const`                                      |
| [`visibility`]                               | Change the visibility of the created builder (defaults to the same visibility as the struct)                | `visibility = pub(crate)`                    |
| [`crate`]                                    | Override the name of the crate when expanding macros (defaults to `bauer`)                                  | `crate = bauer_renamed`                      |
| [`error`]                                    | Set details about the generated error enum (`attributes`, `doc`, `rename`, `force`)                         | `error(...)`                                 |
| [`build_fn`]                                 | Set details about the build function (`attributes`, `doc`, `rename`, `map`)                                 | `build_fn(...)`                              |
| [`builder_fn`]                               | Set details about the builder function added to the struct (`attributes`, `doc`, `rename`)                  | `builder_fn(...)`                            |
| [`attribute`/`attributes`]                   | Set attribute(s) on the generated builder struct                                                            | `attribute(#[foo])`                          |
| [`doc`/`docs`]                               | Set documentation items on the generated builder struct                                                     | `doc(<doc strings>)`                         |

[`kind`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#kind
[`const`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#const
[`visibility`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#visibility
[`crate`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#crate
[`error`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#error
[`attribute`/`attributes`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#attributes
[`doc`/`docs`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#doc
[`build_fn`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#build_fn
[`builder_fn`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#builder_fn

#### Field Configuration

Attributes affect all functions generated the fields in this struct

| Attribute                                    | Description                                                                                                 | Usage                                        |
| -------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------- |
| [`prefix`/`suffix`]                          | Add a prefix/suffix to all field functions created for this builder                                         | `prefix = "set_"` or `suffix = "_field"`     |
| [`on`]                                       | Apply field attributes to fields that match a specific type pattern                                         | `on(<type> => <attributes ...>)`             |

[`prefix`/`suffix`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#prefixsuffix
[`on`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#on

### Field Attributes

All of the attributes that may be applied to fields are listed below.  These go inside of a
`#[builder(..)]` attribute on any field in the struct.  For a more detailed description and
examples, check out the [`Builder`] or click on the attribute.

#### Special Field Treatment

Attributes that affect how a field is treated by the builder

|   Attribute                            | Description                                                                                                 | Usage                              |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| [`associated`]                         | Mark this field as associated data with the builder.  Associated fields are specified by the `builder_fn`.  | `associated`                       |
| [`skip`]                               | Skip this field in the builder and construct its value using the other fields (or default value)            | `skip` or `skip = <value>`         |

[`associated`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#associated
[`skip`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#skip

#### Field Configuration

Attributes that affect how fields are presented to the user

|   Attribute                            | Description                                                                                                 | Usage                              |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| [`rename`]                             | Rename the function that is generated for the field                                                         | `rename = <name>`                  |
| [`skip_prefix`/`skip_suffix`]          | Skip using the prefix/suffix from the builder attribute                                                     | `skip_prefix` or `skip_suffix`     |
| [`visibility`][field_vis]              | Change visibility of the generated function (defaults to the same visibility as the builder)                | `visibility = pub(crate)`          |
| [`attribute`/`attributes`][field_attr] | Set attribute(s) on the function generated for this field                                                   | `attribute(#[foo])`                |
| [`doc`/`docs`][field_doc]              | Set documentation items on the function generated for this field                                            | `doc(<doc strings>)`               |

[`rename`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#rename
[`skip_prefix`/`skip_suffix`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#skip_prefixskip_suffix
[field_vis]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#visibility-1
[field_attr]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#attributes-1
[field_doc]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#doc-1

#### Type Configuration

Configure how the field's type is constructed or specified by the user

|   Attribute                            | Description                                                                                                 | Usage                              |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| [`adapter`]                            | Fully customise how functions take arguments and convert them into the field value                          | `adapter = \|<arg>: <ty>\| <expr>` |
| [`default`]                            | Specify a default value or use [`Default`]                                                                  | `default` or `default = <value>`   |
| [`required`]                           | Make an `Option` field required to be specified as `Some(value)` or `None`                                  | `required`                         |
| [`flag`]                               | Mark a boolean field as a flag, meaning that the if the function is called, the field will be marked true   | `flag`                             |
| [`into`]                               | Make functions accept `impl `[`Into`]`<Field>`                                                              | `into`                             |
| [`tuple`]                              | Make functions accept tuple items as separate arguments                                                     | `tuple` or `tuple(x, y)`           |
| [`repeat`]                             | Allow repeating call to add items to a structure                                                            | `repeat` or `repeat = <type>`      |
| [`repeat_n`]                           | Control the number times a `repeat` field is allowed to be set.  This controls the length of the final data | `repeat_n = 1..` or `repeat_n = 4` |
| [`collector`]                          | Use a custom collector for converting into the target data structure (default: [`FromIterator::from_iter`]) | `collector = <function>`           |

[`adapter`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#adapter
[`default`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#default
[`required`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#required
[`flag`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#flag
[`into`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#into
[`tuple`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#tuple
[`repeat`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#repeat
[`repeat_n`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#repeat_n
[`collector`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html#collector

<!-- cargo-rdme end -->

[`Builder`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html

## Testing

All tests except for the UI tests should work on any rust version and
can simply be run with

```sh
cargo test
```

If you're using rust 1.85 (or `+1.85`), then the UI tests will also be
run.

To specifically run the UI tests, use

```sh
cargo +1.85 test --test ui
```

and if a change to the UI tests has been made, update the values with

```sh
TRYBUILD=overwrite cargo +1.85 test --test ui
```
