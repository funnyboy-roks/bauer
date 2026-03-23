use bauer::Builder;

#[derive(Builder)]
pub struct Foo {
    #[builder(into, adapter(|x: u32, y: u32| (x + 1, y + 1)))]
    field_a: (u32, u32),
}

fn main() {}
