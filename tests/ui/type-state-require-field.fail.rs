use bauer::Builder;

#[derive(Builder)]
#[builder(kind = "type-state")]
struct Foo {
    field_a: u32,
}

fn main() {
    let foo: Foo = Foo::builder().build(); // fail
}
