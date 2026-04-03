#![allow(dead_code)]

// enusre that the same functionality is generated for all kinds (complex)

use bauer::Builder;

macro_rules! define {
    ($name: ident, $kind: literal) => {
        #[derive(Debug, Builder, PartialEq)]
        #[builder(kind = $kind)]
        pub struct $name {
            pub field_a: u32,
            #[builder(default)]
            pub field_b: u32,
            #[builder(default = "42")]
            pub field_c: u32,
            #[builder(default, into)]
            pub field_d: String,
            #[builder(default = "\"hello\"", into)]
            pub field_e: String,
            #[builder(into)]
            pub field_f: String,
            #[builder(repeat)]
            pub field_g: Vec<u32>,
            #[builder(repeat, rename = "field_h_single")]
            pub field_h: Vec<u32>,
            #[builder(repeat, repeat_n = 1..=3)]
            pub field_i: Vec<u32>,
            #[builder(repeat = char)]
            pub field_j: String,
        }
    };
}

macro_rules! populate {
    ($name: ident) => {
        $name::builder()
            .field_a(5)
            .field_f("world")
            .field_g(0)
            .field_g(1)
            .field_h_single(2)
            .field_h_single(3)
            .field_i(4)
            .field_i(5)
            .field_i(6)
            .field_j('h')
            .field_j('i')
    };
}

macro_rules! expected {
    ($name: ident) => {
        $name {
            field_a: 5,
            field_b: 0,
            field_c: 42,
            field_d: String::from(""),
            field_e: String::from("hello"),
            field_f: String::from("world"),
            field_g: vec![0, 1],
            field_h: vec![2, 3],
            field_i: vec![4, 5, 6],
            field_j: String::from("hi"),
        }
    };
}

#[test]
fn complex_owned() {
    define!(Foo, "owned");
    let x = populate!(Foo).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}

#[test]
fn complex_borrowed() {
    define!(Foo, "borrowed");
    let x = populate!(Foo).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}

#[test]
fn complex_type_state() {
    define!(Foo, "type-state");
    let x = populate!(Foo).build();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}
