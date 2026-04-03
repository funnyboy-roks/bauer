#[derive(Debug, bauer::Builder)]
struct Foo {
    #[builder(into, default)]
    field: String,
}

fn main() {
    let x: Foo = Foo::builder().field("hello").build();
    assert_eq!(x.field, "hello");
}
