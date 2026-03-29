#![allow(dead_code)]

use bauer::Builder;

#[test]
fn simple_owned() {
    #[derive(Debug, Builder)]
    #[builder(kind = "owned", prefix = "set_")]
    struct Foo {
        /// Hello
        #[builder(default = "42")]
        field_a: u32,
        field_b: bool,
        #[builder(into)]
        field_c: String,
        #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat, repeat_n = 3..)]
        field_d: Vec<f64>,
    }

    let x: Foo = Foo::builder()
        .set_field_a(69)
        .set_field_b(true)
        .set_field_c("hello world")
        .add_d(std::f64::consts::PI)
        .add_d(std::f64::consts::TAU)
        .add_d(2.72)
        .build()
        .unwrap();

    dbg!(x);
}

#[test]
fn simple_borrowed() {
    #[derive(Debug, Builder)]
    #[builder(kind = "borrowed", prefix = "set_")]
    struct Foo {
        /// Hello
        #[builder(default = "42")]
        field_a: u32,
        field_b: bool,
        #[builder(into)]
        field_c: String,
        #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat, repeat_n = 3..)]
        field_d: Vec<f64>,
    }

    let x: Foo = Foo::builder()
        .set_field_a(69)
        .set_field_b(true)
        .set_field_c("hello world")
        .add_d(std::f64::consts::PI)
        .add_d(std::f64::consts::TAU)
        .add_d(2.72)
        .build()
        .unwrap();

    dbg!(x);
}

#[test]
fn simple_type_state() {
    #[derive(Debug, Builder)]
    #[builder(kind = "type-state", prefix = "set_")]
    struct Foo {
        /// Hello
        #[builder(default = "42")]
        field_a: u32,
        field_b: bool,
        #[builder(into)]
        field_c: String,
        #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat, repeat_n = 3..)]
        field_d: Vec<f64>,
    }

    let x: Foo = Foo::builder()
        .set_field_a(69)
        .set_field_b(true)
        .set_field_c("hello world")
        .add_d(std::f64::consts::PI)
        .add_d(std::f64::consts::TAU)
        .add_d(2.72)
        .build();

    dbg!(x);
}
