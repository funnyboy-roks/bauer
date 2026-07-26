#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct Foo {
                #[builder(required)]
                req: Option<u32>,
            }

            #[test]
            fn some() {
                let f: Foo = Foo::builder()
                    .req(Some(69))
                    .build()
                    $(.$unwrap())?;

                assert_eq!(f.req, Some(69));
            }

            #[test]
            fn none() {
                let f: Foo = Foo::builder()
                    .req(None)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(f.req, None);
            }

            $( // only do the error test for owned/borrowed
                const _: &str = stringify!($unwrap);
                #[test]
                fn missing_error() {
                    let f: Result<Foo, FooBuildError> = Foo::builder()
                        .build();

                    assert_eq!(f, Err(FooBuildError::MissingReq));
                }
            )?
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
