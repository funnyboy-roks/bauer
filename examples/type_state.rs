#![allow(unused)]

use bauer::Builder;

#[derive(Debug, Builder)]
#[builder(kind = "type-state")]
struct Point {
    x: u64,
    y: u64,
    z: u64,
}

#[derive(Debug, Builder)]
#[builder(kind = "type-state")]
struct Foo {
    field_a: u32,
    #[builder(into)]
    other: Point,
}

fn main() {
    let x: Foo = FooBuilder::new()
        .field_a(69)
        .other(Point::builder().x(4).y(2).z(0))
        .into();
    dbg!(x);
}
