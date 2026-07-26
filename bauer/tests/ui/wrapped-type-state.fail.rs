#[derive(bauer::Builder)]
#[builder(kind = "type-state")]
struct Foo {
    #[builder(required)]
    field: Option<u32>,
}

fn main() {
    Foo::builder().build(); // fail -- field not set
}
