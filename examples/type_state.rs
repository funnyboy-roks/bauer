use bauer::Builder;

#[derive(Debug, Builder)]
#[builder(kind = "type-state")]
pub struct Foo {
    pub field_a: Option<String>,
    #[builder(repeat)]
    pub field_b: Vec<u32>,
}

fn main() {
    let x = FooBuilder::new().field_b(69).build();
    dbg!(x);
}
