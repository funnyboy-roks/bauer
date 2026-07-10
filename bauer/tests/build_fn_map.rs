#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            struct OtherType {
                x: u32,
                y: u32,
            }

            impl OtherType {
                fn new(x: u32, y: u32) -> Self {
                    Self { x, y }
                }
            }

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            #[builder(build_fn(
                map = |f| -> OtherType { OtherType::new(f.x, f.y) }
            ))]
            struct Foo {
                x: u32,
                y: u32,
            }

            #[test]
            fn test() {
                let x: OtherType = Foo::builder()
                    .x(69)
                    .y(420)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(x.x, 69);
                assert_eq!(x.y, 420);
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
