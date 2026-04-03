use bauer::Builder;

#[derive(Builder)]
struct Foo {
    #[builder(adapter = |x: u32| char::from_u32(x))]
    field: char,
}

fn main() {
    let _: Foo = Foo::builder().field(6).build().unwrap();
}
