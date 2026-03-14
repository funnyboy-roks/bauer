#[derive(Debug, bauer::Builder)]
struct Foo {
    #[builder(into, default)]
    field: String,
}

fn main() {
    let _foo: Foo = Foo::builder().field("hello").build();
}
