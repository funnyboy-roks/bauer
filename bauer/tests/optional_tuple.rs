#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct OptionalTuple {
                #[builder(tuple)]
                bar: Option<(u32, u32)>,
                #[builder(tuple(a, b))]
                named: Option<(i32, i32)>,
            }

            #[test]
            fn optional_tuple_unset_is_none() {
                let f: OptionalTuple = OptionalTuple::builder().build();
                assert_eq!(f.bar, None);
                assert_eq!(f.named, None);
            }

            #[test]
            fn optional_tuple_set_with_separate_args() {
                let f: OptionalTuple = OptionalTuple::builder()
                    .bar(1, 2)
                    .named(3, 4)
                    .build();

                assert_eq!(f.bar, Some((1, 2)));
                assert_eq!(f.named, Some((3, 4)));
            }
        }
    };
}

tests!("borrowed" in mod borrowed);
tests!("owned" in mod owned);
tests!("type-state" in mod type_state);
