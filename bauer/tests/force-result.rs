#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $error: path) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind, prefix = "set_", error(force))]
            struct Struct {
                /// Hello
                #[builder(default = "42")]
                field_a: u32,
                #[builder(default)]
                field_b: bool,
                #[builder(default, into)]
                field_c: String,
                #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat)]
                field_d: Vec<f64>,
            }

            #[test]
            fn it_works() {
                let Ok(s): Result<Struct, $error> = Struct::builder()
                    .set_field_a(69)
                    .set_field_b(true)
                    .set_field_c("hello world")
                    .add_d(std::f64::consts::PI)
                    .add_d(std::f64::consts::TAU)
                    .add_d(2.72)
                    .build();

                assert_eq!(s.field_a, 69);
                assert_eq!(s.field_b, true);
                assert_eq!(s.field_c, "hello world");
                assert_eq!(
                    s.field_d,
                    [std::f64::consts::PI, std::f64::consts::TAU, 2.72]
                );
            }
        }
    };
}

tests!("borrowed"   in mod   borrowed         StructBuildError);
tests!("owned"      in mod      owned         StructBuildError);
tests!("type-state" in mod type_state std::convert::Infallible);
