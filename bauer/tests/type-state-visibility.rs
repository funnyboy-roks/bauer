mod my_mod {
    use bauer::Builder;
    #[derive(Builder)]
    #[builder(kind = "type-state")]
    pub struct Foo {
        field: u32,
        #[builder(repeat)]
        multiple: [u32; 2],
    }

    impl Foo {
        pub fn field(&self) -> u32 {
            self.field
        }

        pub fn multiple(&self) -> [u32; 2] {
            self.multiple
        }
    }
}

#[test]
fn type_state_visiblity() {
    let b: my_mod::Foo = my_mod::Foo::builder()
        .field(6)
        .multiple(1)
        .multiple(2)
        .build();
    assert_eq!(b.field(), 6);
    assert_eq!(b.multiple(), [1, 2]);
}
