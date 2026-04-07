#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind, build_fn(rename = "finish"))]
            struct Foo {
                #[builder(default = "42")]
                field_a: u32,
                field_b: Option<char>,
                field_c: bool,
                #[builder(into)]
                field_d: String,
                #[builder(skip_prefix, repeat, repeat_n = 3..)]
                field_e: Vec<f64>,
            }

            #[test]
            fn it_works() {
                let f: Foo = Foo::builder()
                    .field_a(69)
                    .field_c(true)
                    .field_d("hello world")
                    .field_e(std::f64::consts::PI)
                    .field_e(std::f64::consts::TAU)
                    .field_e(2.72)
                    .finish()
                    $(.$unwrap())?;

                assert_eq!(f.field_a, 69);
                assert_eq!(f.field_b, None);
                assert_eq!(f.field_c, true);
                assert_eq!(f.field_d, "hello world");
                assert_eq!(f.field_e, [std::f64::consts::PI, std::f64::consts::TAU, 2.72]);
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
