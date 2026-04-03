use bauer::Builder;

#[derive(Builder)]
pub struct Foo {
    #[builder(tuple(a, b))]
    field_a: (i32, i32, i32),
}

fn main() {}
