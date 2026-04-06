#![allow(dead_code)]

// enusre that the same functionality is generated for all kinds (simple)

use std::convert::Infallible;

use bauer::Builder;

macro_rules! define {
    ($name: ident, $kind: literal) => {
        // Normally this builder is infallible
        #[derive(Debug, Builder, PartialEq)]
        #[builder(kind = $kind, prefix = "set_", force_result)]
        struct $name {
            /// Hello
            #[builder(default = "42")]
            field_a: u32,
            #[builder(default)]
            field_b: bool,
            #[builder(default, into)]
            field_c: String,
            #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat)]
            field_d: Vec<f64>,
        }
    };
}

macro_rules! populate {
    ($name: ident) => {
        $name::builder()
            .set_field_a(69)
            .set_field_b(true)
            .set_field_c("hello world")
            .add_d(std::f64::consts::PI)
            .add_d(std::f64::consts::TAU)
            .add_d(2.72)
    };
}

macro_rules! expected {
    ($name: ident) => {
        $name {
            field_a: 69,
            field_b: true,
            field_c: "hello world".into(),
            field_d: vec![std::f64::consts::PI, std::f64::consts::TAU, 2.72],
        }
    };
}

#[test]
fn simple_owned() {
    define!(Foo, "owned");
    let Ok(x): Result<_, Infallible> = populate!(Foo).build();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}

#[test]
fn simple_borrowed() {
    define!(Foo, "borrowed");
    let Ok(x): Result<_, Infallible> = populate!(Foo).build();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}

#[test]
fn simple_type_state() {
    define!(Foo, "type-state");
    let Ok(x): Result<_, Infallible> = populate!(Foo).build();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}
