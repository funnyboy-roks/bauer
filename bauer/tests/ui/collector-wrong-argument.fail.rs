use bauer::Builder;

fn count(_wrong_argument: Vec<u8>) -> usize {
    todo!()
}

#[derive(Builder)]
#[builder(kind = "owned")]
pub struct Foo {
    #[builder(repeat = u8, collector = count)]
    field_a: usize,
}

#[derive(Builder)]
#[builder(kind = "type-state")]
pub struct FooTypeState {
    #[builder(repeat = u8, collector = count)]
    field_a: usize,
}

fn main() {}
