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
    #[builder(repeat)]
    field_a: [u8; 3],
    #[builder(into)]
    other: Point,
}

fn main() {
    let x: Foo = FooBuilder::new()
        .field_a(0)
        .field_a(1)
        .field_a(2)
        .other(Point::builder().x(4).y(2).z(0))
        .into();
    dbg!(x);
}
