use bauer::Builder;

#[derive(Builder)]
pub struct Foo {
    #[builder(repeat, repeat_n = 3)]
    field_a: [u32; 3],
}

fn main() {}
