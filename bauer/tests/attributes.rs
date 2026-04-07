#[rustfmt::skip] // rustfmt really does not like this style of attributes in macro
macro_rules! tests {
    ($kind: literal in mod $module: ident $(unwrap:$unwrap: ident)? $(error:$error: ident)?) => {
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
                    fn builder_attribute_set() {
                        assert!(Struct::BUILDER_ATTRIBUTE_SET);
                    }

                    $(
                        #[test]
                        fn error_attribute_set() {
                            let _ = stringify!($error); // just to get the repeat working
                            assert!(ERROR_ATTRIBUTE_SET);
                        }
                    )?

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
                    build_fn(
                        attributes(
                            #[attribute::pre(
                                const BUILD_ATTRIBUTE_SET: bool = true;
                            )]
                        )
                    ),
                    builder_fn(
                        attributes(
                            #[attribute::pre(
                                const BUILDER_ATTRIBUTE_SET: bool = true;
                            )]
                        )
                    ),
                    $($error(
                        attributes(
                            #[attribute::pre(
                                const ERROR_ATTRIBUTE_SET: bool = true;
                            )]
                        )
                    ),)?
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
                    build_fn {
                        attributes {
                            #[attribute::pre(
                                const BUILD_ATTRIBUTE_SET: bool = true;
                            )]
                        }
                    },
                    builder_fn {
                        attributes {
                            #[attribute::pre(
                                const BUILDER_ATTRIBUTE_SET: bool = true;
                            )]
                        }
                    },
                    $($error {
                        attributes {
                            #[attribute::pre(
                                const ERROR_ATTRIBUTE_SET: bool = true;
                            )]
                        }
                    },)?
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

tests!("borrowed" in mod borrowed unwrap:unwrap error:error);
tests!("owned" in mod owned unwrap:unwrap error:error);
tests!("type-state" in mod type_state);
