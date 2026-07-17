#![allow(dead_code)]
#![allow(clippy::disallowed_names)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {

            mod all_default {
                use bauer::Builder;
                use std::collections::HashMap;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind, on(_ => default))]
                struct Foo {
                    u32: u32,
                    vec: Vec<u32>,
                    map: HashMap<u32, u32>,
                }

                #[test]
                fn minimal() {
                    let c = Foo::builder().build();

                    assert_eq!(c.u32, 0);
                    assert_eq!(c.vec, [0; 0]);
                    assert_eq!(c.map, HashMap::new());
                }

                #[test]
                fn full() {
                    let c = Foo::builder()
                        .u32(42)
                        .vec(vec![123, 456])
                        .map([(1, 2), (3, 4)].into())
                        .build();

                    assert_eq!(c.u32, 42);
                    assert_eq!(c.vec, [123, 456]);
                    assert_eq!(c.map, [(1, 2), (3, 4)].into());
                }
            }

            mod vec_repeat {
                use bauer::Builder;
                use std::collections::HashMap;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind, on(Vec<_> => repeat))]
                struct Foo {
                    #[builder(default)]
                    u32: u32,
                    vec: Vec<u32>,
                    vec2: Vec<u32>,
                    #[builder(default)]
                    map: HashMap<u32, u32>,
                }

                #[test]
                fn minimal() {
                    let c = Foo::builder().build();

                    assert_eq!(c.u32, 0);
                    assert_eq!(c.vec, [0; 0]);
                    assert_eq!(c.vec2, [0; 0]);
                    assert_eq!(c.map, HashMap::new());
                }

                #[test]
                fn full() {
                    let c = Foo::builder()
                        .u32(42)
                        .vec(123)
                        .vec(456)
                        .vec2(789)
                        .map([(1, 2), (3, 4)].into())
                        .build();

                    assert_eq!(c.u32, 42);
                    assert_eq!(c.vec, [123, 456]);
                    assert_eq!(c.vec2, [789]);
                    assert_eq!(c.map, [(1, 2), (3, 4)].into());
                }
            }

            mod vec_map_repeat {
                use bauer::Builder;
                use std::collections::HashMap;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind, on(Vec<_> => repeat), on(HashMap<_, _> => repeat = (#0, #1), tuple))]
                struct Foo {
                    #[builder(default)]
                    u32: u32,
                    vec: Vec<u32>,
                    map: HashMap<u32, u32>,
                }

                #[test]
                fn minimal() {
                    let c = Foo::builder().build();

                    assert_eq!(c.u32, 0);
                    assert_eq!(c.vec, [0; 0]);
                    assert_eq!(c.map, HashMap::new());
                }

                #[test]
                fn full() {
                    let c = Foo::builder()
                        .u32(42)
                        .vec(123)
                        .vec(456)
                        .map(1, 2)
                        .map(3, 4)
                        .build();

                    assert_eq!(c.u32, 42);
                    assert_eq!(c.vec, [123, 456]);
                    assert_eq!(c.map, [(1, 2), (3, 4)].into());
                }
            }

            mod automatic_manual_tuple {
                use bauer::Builder;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind, on((_, _) => adapter = |a: #0, b: #1| (a, b)))]
                struct Foo<'a> {
                    foo: (u32, &'a str)
                }

                #[test]
                fn test() {
                    let c = Foo::builder()
                        .foo(69, "hello")
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(c.foo, (69, "hello"));
                }
            }

            mod empty {
                use bauer::Builder;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind, on(_ => ))]
                struct Foo {
                    x: u32
                }

                #[test]
                fn test() {
                    let c = Foo::builder()
                        .x(69)
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(c.x, 69);
                }
            }

            mod string_into {
                use bauer::Builder;

                #[derive(Debug, Builder, PartialEq)]
                #[builder(kind = $kind, on(String => into))]
                struct Foo {
                    x: String,
                    y: String,
                    z: String,
                }

                #[test]
                fn test() {
                    let c = Foo::builder()
                        .x("hello x")
                        .y("hello y")
                        .z("hello z")
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(c.x, "hello x");
                    assert_eq!(c.y, "hello y");
                    assert_eq!(c.z, "hello z");
                }
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
