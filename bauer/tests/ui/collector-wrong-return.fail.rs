use bauer::Builder;

fn count(_iter: impl Iterator<Item = u8>) -> Vec<u8> {
    todo!()
}

#[derive(Builder)]
#[builder(kind = "owned")]
pub struct FooOwned {
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
