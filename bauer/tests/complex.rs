#![allow(dead_code)]

fn sum(iter: impl Iterator<Item = u32>) -> u32 {
    iter.sum()
}

fn add_2(iter: impl Iterator<Item = u32>) -> Vec<u32> {
    iter.map(|n| n + 2).collect()
}

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct Complex {
                required: u32,
                optional: Option<u32>,
                #[builder(default)]
                default: u32,
                #[builder(default = "42")]
                default_value: u32,
                #[builder(repeat)]
                repeat: Vec<u32>,
                #[builder(repeat = char)]
                repeat_ty: String,
                #[builder(repeat, repeat_n = 3)]
                repeat_n_exact: Vec<u32>,
                #[builder(repeat, repeat_n = 3..)]
                repeat_n_at_least: Vec<u32>,
                #[builder(repeat, repeat_n = ..5)]
                repeat_n_at_most: Vec<u32>,
                #[builder(repeat, repeat_n = 2..5)]
                repeat_n_range_ex: Vec<u32>,
                #[builder(repeat, repeat_n = 2..=5)]
                repeat_n_range_in: Vec<u32>,
                #[builder(repeat, collector = super::add_2)]
                collector_map: Vec<u32>,
                #[builder(repeat = u32, collector = super::sum)]
                collector_sum: u32,
                #[builder(into)]
                into: String,
                #[builder(into, repeat)]
                into_repeat: Vec<String>,
                #[builder(tuple)]
                tuple: (u32, u32),
                #[builder(tuple, into)]
                tuple_into: (String, String),
                #[builder(repeat, tuple)]
                tuple_repeat: Vec<(u32, u32)>,
                #[builder(repeat, tuple, into)]
                tuple_repeat_into: Vec<(String, String)>,
                #[builder(rename = "renamed")]
                not_renamed: u32,
            }

            #[test]
            fn minimal() {
                let c: Complex = Complex::builder()
                    .required(69)
                    .repeat_n_exact(1)
                    .repeat_n_exact(2)
                    .repeat_n_exact(3)
                    .repeat_n_at_least(1)
                    .repeat_n_at_least(2)
                    .repeat_n_at_least(3)
                    .repeat_n_range_ex(1)
                    .repeat_n_range_ex(2)
                    .repeat_n_range_in(1)
                    .repeat_n_range_in(2)
                    .into("into")
                    .tuple(8, 9)
                    .tuple_into("a", "b")
                    .renamed(4)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(c.required, 69);
                assert_eq!(c.optional, None);
                assert_eq!(c.default, 0);
                assert_eq!(c.default_value, 42);
                assert_eq!(c.repeat, []);
                assert_eq!(c.repeat_ty, "");
                assert_eq!(c.repeat_n_exact, [1, 2, 3]);
                assert_eq!(c.repeat_n_at_least, [1, 2, 3]);
                assert_eq!(c.repeat_n_at_most, []);
                assert_eq!(c.repeat_n_range_ex, [1, 2]);
                assert_eq!(c.repeat_n_range_in, [1, 2]);
                assert_eq!(c.collector_map, []);
                assert_eq!(c.collector_sum, 0);
                assert_eq!(c.into, "into");
                assert_eq!(c.into_repeat, <[&str; 0]>::default());
                assert_eq!(c.tuple, (8, 9));
                assert_eq!(c.tuple_into, ("a".to_string(), "b".to_string()));
                assert_eq!(c.tuple_repeat, []);
                assert_eq!(c.tuple_repeat_into, []);
                assert_eq!(c.not_renamed, 4);
            }

            #[test]
            fn full() {
                let c: Complex = Complex::builder()
                    .required(69)
                    .optional(8675309)
                    .default(42)
                    .default_value(1337)
                    .repeat(1)
                    .repeat(2)
                    .repeat_ty('h')
                    .repeat_ty('i')
                    .repeat_n_exact(1)
                    .repeat_n_exact(2)
                    .repeat_n_exact(3)
                    .repeat_n_at_least(1)
                    .repeat_n_at_least(2)
                    .repeat_n_at_least(3)
                    .repeat_n_at_most(1)
                    .repeat_n_at_most(2)
                    .repeat_n_at_most(3)
                    .repeat_n_range_ex(1)
                    .repeat_n_range_ex(2)
                    .repeat_n_range_in(1)
                    .repeat_n_range_in(2)
                    .collector_map(1)
                    .collector_map(2)
                    .collector_map(3)
                    .collector_sum(1)
                    .collector_sum(2)
                    .collector_sum(3)
                    .into("into")
                    .into_repeat("a")
                    .into_repeat("b")
                    .into_repeat("c")
                    .tuple(8, 9)
                    .tuple_into("a", "b")
                    .tuple_repeat(1, 2)
                    .tuple_repeat(3, 4)
                    .tuple_repeat_into("a", "b")
                    .tuple_repeat_into("c", "d")
                    .renamed(4)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(c.required, 69);
                assert_eq!(c.optional, Some(8675309));
                assert_eq!(c.default, 42);
                assert_eq!(c.default_value, 1337);
                assert_eq!(c.repeat, [1, 2]);
                assert_eq!(c.repeat_ty, "hi");
                assert_eq!(c.repeat_n_exact, [1, 2, 3]);
                assert_eq!(c.repeat_n_at_least, [1, 2, 3]);
                assert_eq!(c.repeat_n_at_most, [1, 2, 3]);
                assert_eq!(c.repeat_n_range_ex, [1, 2]);
                assert_eq!(c.repeat_n_range_in, [1, 2]);
                assert_eq!(c.collector_map, [3, 4, 5]);
                assert_eq!(c.collector_sum, 1 + 2 + 3);
                assert_eq!(c.into, "into");
                assert_eq!(c.into_repeat, ["a", "b", "c"]);
                assert_eq!(c.tuple, (8, 9));
                assert_eq!(c.tuple_into, ("a".to_string(), "b".to_string()));
                assert_eq!(c.tuple_repeat, [(1, 2), (3, 4)]);
                assert_eq!(c.tuple_repeat_into, [
                    ("a".to_string(), "b".to_string()),
                    ("c".to_string(), "d".to_string())
                ]);
                assert_eq!(c.not_renamed, 4);
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
