use bauer::Builder;

#[derive(Builder)]
#[builder(on(_ => foo))]
pub struct Foo {
    field_a: u32,
}

fn main() {}
