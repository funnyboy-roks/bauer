#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct Foo {
                no_repeat: [char; 3],
                #[builder(repeat)]
                repeat: [u32; 3],
                #[builder(repeat, into)]
                repeat_into: [String; 3],
                #[builder(repeat, tuple)]
                repeat_tuple: [(u8, u8); 3],
                #[builder(repeat, adapter = |x: u32| x + 2)]
                repeat_adapter: [u32; 4],
            }

            #[test]
            fn test() {
                let x: Foo = Foo::builder()
                    .no_repeat(['a', 'b', 'c'])
                    .repeat(0)
                    .repeat(1)
                    .repeat(2)
                    .repeat_into("foo")
                    .repeat_into("bar")
                    .repeat_into("baz")
                    .repeat_tuple(1, 6)
                    .repeat_tuple(2, 5)
                    .repeat_tuple(3, 4)
                    .repeat_adapter(6)
                    .repeat_adapter(12)
                    .repeat_adapter(18)
                    .repeat_adapter(24)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(x.no_repeat, ['a', 'b', 'c']);
                assert_eq!(x.repeat, [0, 1, 2]);
                assert_eq!(x.repeat_into, ["foo", "bar", "baz"]);
                assert_eq!(x.repeat_tuple, [(1, 6), (2, 5), (3, 4)]);
                assert_eq!(x.repeat_adapter, [8, 14, 20, 26]);
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
