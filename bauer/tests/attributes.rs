#[rustfmt::skip] // rustfmt really does not like this style of attributes in macro
macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        pub mod $module {
            macro_rules! test {
                () => {
                    #[test]
                    fn struct_attribute_set() {
                        assert!(STRUCT_ATTRIBUTE_SET);
                    }

                    #[test]
                    fn build_attribute_set() {
                        assert!(StructBuilder::BUILD_ATTRIBUTE_SET);
                    }

                    #[test]
                    fn field_attribute_set() {
                        assert!(StructBuilder::FIELD_ATTRIBUTE_SET);
                    }

                    #[test]
                    fn build() {
                        let x = Struct::builder().field(0).build().unwrap();
                        assert_eq!(x.field, 0);
                    }
                }
            }

            /// Ensure that () form of attributes works
            mod paren {
                use bauer::Builder;

                #[derive(Builder)]
                #[builder(
                    attributes(
                        #[attribute::pre(
                            static STRUCT_ATTRIBUTE_SET: bool = true;
                        )]
                    ),
                    build_fn_attributes(
                        #[attribute::pre(
                            const BUILD_ATTRIBUTE_SET: bool = true;
                        )]
                    ),
                )]
                pub struct Struct {
                    #[builder(
                        attributes(
                            #[attribute::pre(
                                const FIELD_ATTRIBUTE_SET: bool = true;
                            )]
                        )
                    )]
                    field: u8,
                }

                test!();
            }

            /// Ensure that {} form of attributes works
            mod brace {
                use bauer::Builder;

                #[derive(Builder)]
                #[builder(
                    attributes {
                        #[attribute::pre(
                            static STRUCT_ATTRIBUTE_SET: bool = true;
                        )]
                    },
                    build_fn_attributes {
                        #[attribute::pre(
                            const BUILD_ATTRIBUTE_SET: bool = true;
                        )]
                    },
                )]
                pub struct Struct {
                    #[builder(
                        attributes {
                            #[attribute::pre(
                                const FIELD_ATTRIBUTE_SET: bool = true;
                            )]
                        }
                    )]
                    field: u8,
                }

                test!();
            }

        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
