use bauer::Builder;

fn count(_iter: impl Iterator<Item = u8>) -> Vec<u8> {
    todo!()
}

#[derive(Builder)]
#[builder(kind = "owned")]
pub struct FooOwnedArrays {
    #[builder(repeat, collector = count)]
    field_a: [u8; 3],
}

#[derive(Builder)]
#[builder(kind = "type-state")]
pub struct FooTypeStateArrays {
    #[builder(repeat, collector = count)]
    field_a: [u8; 3],
}

#[derive(Builder)]
#[builder(kind = "owned")]
pub struct FooOwnedNoRepeat {
    #[builder(collector = count)]
    field_a: Vec<u8>,
}

#[derive(Builder)]
#[builder(kind = "type-state")]
pub struct FooTypeStateNoRepeat {
    #[builder(collector = count)]
    field_a: Vec<u8>,
}

fn main() {}
