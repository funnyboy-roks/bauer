#![allow(dead_code)]

use bauer::Builder;

macro_rules! define {
    ($name: ident, $kind: literal) => {
        #[derive(Debug, Builder, PartialEq)]
        #[builder(kind = $kind, const)]
        struct $name {
            required: u32,
            #[builder(default = "42")]
            default_u32: u32,
            #[builder(default = "\"hello\"")]
            default_str: &'static str,
            #[builder(repeat)]
            array: [u32; 3],
            #[builder(adapter = |b: u8| char::from_u32(b as u32).unwrap())]
            adapter: char,
            #[builder(rename = "renamed_field")]
            rename: char,
            #[builder(tuple)]
            tuple: (u8, u8),
        }
    };
}

macro_rules! populate {
    ($name: ident with defaults) => {
        $name::builder()
            .required(5)
            .array(4)
            .array(2)
            .array(0)
            .adapter(69)
            .renamed_field('g')
            .tuple(4, 2)
    };
    ($name: ident with full) => {
        $name::builder()
            .required(5)
            .default_u32(123)
            .default_str("not hello")
            .array(4)
            .array(2)
            .array(0)
            .adapter(69)
            .renamed_field('g')
            .tuple(4, 2)
    };
}

macro_rules! expected {
    ($name: ident with defaults) => {
        $name {
            required: 5,
            default_u32: 42,
            default_str: "hello",
            array: [4, 2, 0],
            adapter: 'E',
            rename: 'g',
            tuple: (4, 2),
        }
    };
    ($name: ident with full) => {
        $name {
            required: 5,
            default_u32: 123,
            default_str: "not hello",
            array: [4, 2, 0],
            adapter: 'E',
            rename: 'g',
            tuple: (4, 2),
        }
    };
}

#[test]
fn konst_owned_defaults() {
    define!(Foo, "owned");
    let x = populate!(Foo with defaults).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo with defaults));
}

#[test]
fn konst_owned_full() {
    define!(Foo, "owned");
    let x = populate!(Foo with full).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo with full));
}

#[test]
fn konst_borrowed_defaults() {
    define!(Foo, "borrowed");
    let x = populate!(Foo with defaults).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo with defaults));
}

#[test]
fn konst_borrowed_full() {
    define!(Foo, "borrowed");
    let x = populate!(Foo with full).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo with full));
}

#[test]
fn konst_type_state_defaults() {
    define!(Foo, "type-state");
    let x = populate!(Foo with defaults).build();
    dbg!(&x);
    assert_eq!(x, expected!(Foo with defaults));
}

#[test]
fn konst_type_state_full() {
    define!(Foo, "type-state");
    let x = populate!(Foo with full).build();
    dbg!(&x);
    assert_eq!(x, expected!(Foo with full));
}
