#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct Foo {
                #[builder(flag)]
                my_flag: bool,
            }

            #[test]
            fn set() {
                let x: Foo = Foo::builder().my_flag().build();

                assert!(x.my_flag);
            }

            #[test]
            fn unset() {
                let x: Foo = Foo::builder().build();

                assert!(!x.my_flag);
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
