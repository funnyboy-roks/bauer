use bauer::Builder;

#[derive(Builder)]
pub struct Foo {
    #[builder(repeat = u8)]
    field_a: [u32; 3],
}

fn main() {}
