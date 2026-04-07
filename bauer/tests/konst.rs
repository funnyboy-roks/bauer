#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind, const)]
            struct Const {
                required: u32,
                #[builder(default = "42")]
                default_u32: u32,
                #[builder(default = "\"hello\"")]
                default_str: &'static str,
                #[builder(repeat)]
                array: [u32; 3],
                #[builder(adapter = |b: u8| char::from_u32(b as u32).unwrap())]
                adapter: char,
                #[builder(rename = "renamed_field")]
                rename: char,
                #[builder(tuple)]
                tuple: (u8, u8),
            }

            #[test]
            fn default() {
                let c: Const = Const::builder()
                    .required(5)
                    .array(4)
                    .array(2)
                    .array(0)
                    .adapter(69)
                    .renamed_field('g')
                    .tuple(4, 2)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(c.required, 5);
                assert_eq!(c.default_u32, 42);
                assert_eq!(c.default_str, "hello");
                assert_eq!(c.array, [4, 2, 0]);
                assert_eq!(c.adapter, 'E');
                assert_eq!(c.rename, 'g');
                assert_eq!(c.tuple, (4, 2));
            }

            // Option::unwrap is not const
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

            #[test]
            fn default_at_const() {
                const C: Const = unwrap!(
                    Const::builder()
                        .required(5)
                        .array(4)
                        .array(2)
                        .array(0)
                        .adapter(69)
                        .renamed_field('g')
                        .tuple(4, 2)
                        .build(),
                    $($unwrap)?
                );

                assert_eq!(C.required, 5);
                assert_eq!(C.default_u32, 42);
                assert_eq!(C.default_str, "hello");
                assert_eq!(C.array, [4, 2, 0]);
                assert_eq!(C.adapter, 'E');
                assert_eq!(C.rename, 'g');
                assert_eq!(C.tuple, (4, 2));
            }

            #[test]
            fn full() {
                let c: Const = Const::builder()
                    .required(5)
                    .default_u32(123)
                    .default_str("not hello")
                    .array(4)
                    .array(2)
                    .array(0)
                    .adapter(69)
                    .renamed_field('g')
                    .tuple(4, 2)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(c.required, 5);
                assert_eq!(c.default_u32, 123);
                assert_eq!(c.default_str, "not hello");
                assert_eq!(c.array, [4, 2, 0]);
                assert_eq!(c.adapter, 'E');
                assert_eq!(c.rename, 'g');
                assert_eq!(c.tuple, (4, 2));
            }

            #[test]
            fn full_at_const() {
                const C: Const = unwrap!(
                    Const::builder()
                        .required(5)
                        .default_u32(123)
                        .default_str("not hello")
                        .array(4)
                        .array(2)
                        .array(0)
                        .adapter(69)
                        .renamed_field('g')
                        .tuple(4, 2)
                        .build(),
                    $($unwrap)?
                );

                assert_eq!(C.required, 5);
                assert_eq!(C.default_u32, 123);
                assert_eq!(C.default_str, "not hello");
                assert_eq!(C.array, [4, 2, 0]);
                assert_eq!(C.adapter, 'E');
                assert_eq!(C.rename, 'g');
                assert_eq!(C.tuple, (4, 2));
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
