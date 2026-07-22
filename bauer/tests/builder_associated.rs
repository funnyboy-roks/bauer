#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            mod single_simple {
                use bauer::Builder;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind)]
                struct Foo {
                    #[builder(associated)]
                    assoc_a: String,
                    field_a: u32,
                }

                #[test]
                fn it_works() {
                    let f: Foo = Foo::builder("Hello world".into())
                        .field_a(69)
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(f.assoc_a, "Hello world");
                    assert_eq!(f.field_a, 69);
                }
            }

            mod single_complex {
                use bauer::Builder;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind)]
                struct Foo {
                    #[builder(associated, into)]
                    assoc_a: String,
                    field_a: u32,
                }

                #[test]
                fn it_works() {
                    let f: Foo = Foo::builder("Hello world")
                        .field_a(69)
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(f.assoc_a, "Hello world");
                    assert_eq!(f.field_a, 69);
                }
            }

            mod multiple_simple {
                use bauer::Builder;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind)]
                struct Foo {
                    #[builder(associated)]
                    assoc_a: String,
                    #[builder(associated)]
                    assoc_b: u32,
                    field_a: u32,
                }

                #[test]
                fn it_works() {
                    let f: Foo = Foo::builder("Hello world".into(), 420)
                        .field_a(69)
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(f.assoc_a, "Hello world");
                    assert_eq!(f.assoc_b, 420);
                    assert_eq!(f.field_a, 69);
                }
            }

            mod multiple_complex {
                use bauer::Builder;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind)]
                struct Foo {
                    #[builder(associated, into)]
                    assoc_a: String,
                    #[builder(associated, rename = "assoc_field_b")]
                    assoc_b: u32,
                    field_a: u32,
                }

                #[test]
                fn it_works() {
                    let f: Foo = Foo::builder("Hello world", 420)
                        .field_a(69)
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(f.assoc_a, "Hello world");
                    assert_eq!(f.assoc_b, 420);
                    assert_eq!(f.field_a, 69);
                }
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
