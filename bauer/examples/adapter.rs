use bauer::Builder;

#[derive(Debug, Builder, PartialEq)]
pub struct Foo {
    // #[builder(adapter(into, tuple(foo, bar)))]
    // #[builder(adapter = into)]
    // #[builder(adapter = tuple(foo, bar))]
    // #[builder(adapter = |foo: impl Into<u32>, bar: impl Into<u32>| (foo.into(), bar.into()) )]
    #[builder(adapter(|x: u32, y: u32| (x + 2, y + 7)))]
    pub field_a: (u32, u32),
}

fn main() {
    let x = Foo::builder().field_a(5, 7).build().unwrap();

    dbg!(&x);
}
