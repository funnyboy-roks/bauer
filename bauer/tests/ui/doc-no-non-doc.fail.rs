use bauer::Builder;

#[derive(Builder)]
#[builder(doc(#[not_doc_attribute = "foo"]))]
struct Foo {
    field_a: u32,
}

fn main() {}
