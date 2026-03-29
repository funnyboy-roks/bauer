<!-- Readme generated with `cargo-readme`: https://github.com/webern/cargo-readme -->

# bauer

[![Crates.io](https://img.shields.io/crates/v/bauer.svg)](https://crates.io/crates/bauer)
[![Documentation](https://docs.rs/bauer/badge.svg)](https://docs.rs/bauer/)
[![Dependency status](https://deps.rs/repo/github/funnyboy-roks/bauer/status.svg)](https://deps.rs/repo/github/funnyboy-roks/bauer)

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

Check out [the repository](https://github.com/funnyboy-roks/bauer/tree/main/examples) for more
examples!

## Configuration

Builders are very configurable.  A few of the biggest features can be found below.  For a more
comprehensive collection of features, look at the [`Builder`] macro.

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

### Field Attributes

These attributes go in `#[builder(..)]` on individual fields of the structure

#### **`default`**

Specify a default value for the field to have, or use [`Default::default`]

#### **`repeat`**

Allow any structure which supports [`FromIterator`] to be specified by calling the function
multiple times.  If `repeat_n` is specified, the number of times to repeat is limited.

#### **`into`**/**`tuple`**/**`adapter`**

Change how the generated builder function handles input.  Can also be used with `repeat`.

- `into` will make the function accepet `impl Into<T>`
- `tuple` will make the function accept each item as a separate argument
- `adapter` can specify each argument and how they should be converted into the value

**There are many more attributes, all can be found on the [`Builder`] macro.**

[`Builder`]: https://docs.rs/bauer/latest/bauer/derive.Builder.html
