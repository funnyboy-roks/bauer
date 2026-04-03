use bauer::Builder;

#[derive(Builder)]
pub struct Foo {
    #[builder(adapter(|x: u32, y: u32| (x + 1, y + 1)), into)]
    field_a: (u32, u32),
}

fn main() {}
