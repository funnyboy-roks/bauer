use bauer::Builder;

#[derive(Debug, Builder, PartialEq)]
#[builder(kind = "type-state", const)]
pub struct Foo {
    pub field_a: u32,
    #[builder(default = "42")]
    pub field_b: u32,
    #[builder(default = "\"hello\"")]
    pub field_c: &'static str,
    #[builder(repeat)]
    pub field_d: [u32; 3],
}

fn main() {
    let x = Foo::builder()
        .field_a(5)
        .field_d(4)
        .field_d(2)
        .field_d(0)
        .build();

    dbg!(&x);

    assert_eq!(
        x,
        Foo {
            field_a: 5,
            field_b: 42,
            field_c: "hello",
            field_d: [4, 2, 0],
        }
    );
}

#[test]
fn test_main() {
    main();
}
