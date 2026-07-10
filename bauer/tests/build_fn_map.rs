#![allow(dead_code)]

macro_rules! unwrap {
    ($r: expr, unwrap) => {
        match $r {
            Ok(t) => t,
            Err(_) => panic!("Error"),
        }
    };
    ($r: expr,) => {
        $r
    };
}

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            mod other_type {
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

            mod konst {
                use bauer::Builder;

                struct OtherType {
                    x: u32,
                    y: u32,
                }

                impl OtherType {
                    const fn new(x: u32, y: u32) -> Self {
                        Self { x, y }
                    }
                }

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind)]
                #[builder(const)]
                #[builder(build_fn(
                    map = |f| -> OtherType { OtherType::new(f.x, f.y) }
                ))]
                struct Foo {
                    x: u32,
                    y: u32,
                }

                #[test]
                fn test() {
                    const X: OtherType = unwrap!(
                         Foo::builder()
                            .x(69)
                            .y(420)
                            .build(),
                        $($unwrap)?
                    );

                    assert_eq!(X.x, 69);
                    assert_eq!(X.y, 420);
                }
            }

            mod function {
                use bauer::Builder;

                fn my_function(x: u32, y: u32) -> u32 {
                    (x + y) / 2
                }

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind)]
                #[builder(build_fn(
                    map = |f| -> u32 { my_function(f.x, f.y) }
                ))]
                struct Foo {
                    x: u32,
                    y: u32,
                }

                #[test]
                fn test() {
                    let x: u32 = Foo::builder()
                        .x(69)
                        .y(420)
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(x, (69 + 420) / 2);
                }
            }

            mod konst_function {
                use bauer::Builder;

                const fn my_function(x: u32, y: u32) -> u32 {
                    (x + y) / 2
                }

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind)]
                #[builder(const)]
                #[builder(build_fn(
                    map = |f| -> u32 { my_function(f.x, f.y) }
                ))]
                struct Foo {
                    x: u32,
                    y: u32,
                }

                #[test]
                fn test() {
                    const X: u32 = unwrap!(
                         Foo::builder()
                            .x(69)
                            .y(420)
                            .build(),
                        $($unwrap)?
                    );

                    assert_eq!(X, (69 + 420) / 2);
                }
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
