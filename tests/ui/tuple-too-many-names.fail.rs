use bauer::Builder;

#[derive(Builder)]
pub struct Foo {
    #[builder(tuple(a, b, c))]
    field_a: (i32, i32),
}

fn main() {}
