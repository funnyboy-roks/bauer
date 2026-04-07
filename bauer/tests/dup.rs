#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        #[attribute::dup(
            [Complex0, complex0],
            [Complex1, complex1],
            [Complex2, complex2],
            [Complex3, complex3],
            [Complex4, complex4],
            [Complex5, complex5],
            [Complex6, complex6],
        )]
        mod $module {
            #[derive(Debug, bauer::Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct NAME_0 {
                field_a: u32,
                #[builder(default)]
                field_b: u32,
                #[builder(default = "42")]
                field_c: u32,
                #[builder(default, into)]
                field_d: String,
                #[builder(default = "\"hello\"", into)]
                field_e: String,
                #[builder(into)]
                field_f: String,
                #[builder(repeat)]
                field_g: Vec<u32>,
                #[builder(repeat, rename = "field_h_single")]
                field_h: Vec<u32>,
                #[builder(repeat, repeat_n = 1..=3)]
                field_i: Vec<u32>,
                #[builder(repeat = char)]
                field_j: String,
            }

            #[test]
            fn NAME_1() {
                let c: NAME_0 = NAME_0::builder()
                    .field_a(5)
                    .field_f("world")
                    .field_g(0)
                    .field_g(1)
                    .field_h_single(2)
                    .field_h_single(3)
                    .field_i(4)
                    .field_i(5)
                    .field_i(6)
                    .field_j('h')
                    .field_j('i')
                    .build()
                    $(.$unwrap())?;

                assert_eq!(c.field_a, 5);
                assert_eq!(c.field_b, 0);
                assert_eq!(c.field_c, 42);
                assert_eq!(c.field_d, "");
                assert_eq!(c.field_e, "hello");
                assert_eq!(c.field_f, "world");
                assert_eq!(c.field_g, [0, 1]);
                assert_eq!(c.field_h, [2, 3]);
                assert_eq!(c.field_i, [4, 5, 6]);
                assert_eq!(c.field_j, "hi");
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
