use bauer::Builder;

#[test]
fn required_option_owned() {
    #[derive(Builder, Debug, PartialEq)]
    struct Foo {
        #[builder(required)]
        field: Option<u32>,
    }

    let foo = Foo::builder()
        .field(Some(42))
        .build()
        .unwrap();
    assert_eq!(foo, Foo { field: Some(42) });

    let foo_none = Foo::builder()
        .field(None)
        .build()
        .unwrap();
    assert_eq!(foo_none, Foo { field: None });

    let err = Foo::builder().build().unwrap_err();
    // The exact error name depends on the field name
    assert!(format!("{:?}", err).contains("MissingField"));
}

#[test]
fn required_option_type_state() {
    #[derive(Builder, Debug, PartialEq)]
    #[builder(kind = "type-state")]
    struct Foo {
        #[builder(required)]
        field: Option<u32>,
    }

    let foo = Foo::builder()
        .field(Some(42))
        .build();
    assert_eq!(foo, Foo { field: Some(42) });

    let foo_none = Foo::builder()
        .field(None)
        .build();
    assert_eq!(foo_none, Foo { field: None });

    // This should fail to compile if I uncomment it
    // let foo_fail = Foo::builder().build();
}
