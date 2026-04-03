use bauer::Builder;

#[derive(Builder)]
pub struct Foo {
    #[builder(tuple, adapter(|x: u32, y: u32| (x + 1, y + 1)))]
    field_b: String,
}

fn main() {}
