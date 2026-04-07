#![allow(dead_code)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            mod all {
                mod single {
                    use bauer::Builder;

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct Skip {
                        #[builder(skip)]
                        field_a: u32,
                    }

                    #[test]
                    fn default() {
                        let f: Skip = Skip::builder().build();

                        assert_eq!(f.field_a, 0);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipValue {
                        #[builder(skip = 42)]
                        field_a: u32,
                    }

                    #[test]
                    fn value() {
                        let f: SkipValue = SkipValue::builder().build();

                        assert_eq!(f.field_a, 42);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipOption {
                        #[builder(skip)]
                        field_a: Option<u32>,
                    }

                    #[test]
                    fn default_option() {
                        let f: SkipOption = SkipOption::builder().build();

                        assert_eq!(f.field_a, None);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipOptionValue {
                        #[builder(skip = Some(42))]
                        field_a: Option<u32>,
                    }

                    #[test]
                    fn value_option() {
                        let f: SkipOptionValue = SkipOptionValue::builder().build();

                        assert_eq!(f.field_a, Some(42));
                    }
                }

                mod multi {
                    use bauer::Builder;

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipValueDefault {
                        #[builder(skip = 42)]
                        field_a: u32,
                        #[builder(skip)]
                        field_b: u32,
                    }

                    #[test]
                    fn value_default() {
                        let f: SkipValueDefault = SkipValueDefault::builder().build();

                        assert_eq!(f.field_a, 42);
                        assert_eq!(f.field_b, 0);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipValue {
                        #[builder(skip = 42)]
                        field_a: u32,
                        #[builder(skip = 24)]
                        field_b: u32,
                    }

                    #[test]
                    fn value() {
                        let f: SkipValue = SkipValue::builder().build();

                        assert_eq!(f.field_a, 42);
                        assert_eq!(f.field_b, 24);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipDefault {
                        #[builder(skip)]
                        field_a: u32,
                        #[builder(skip)]
                        field_b: u32,
                    }

                    #[test]
                    fn default() {
                        let f: SkipDefault = SkipDefault::builder().build();

                        assert_eq!(f.field_a, 0);
                        assert_eq!(f.field_b, 0);
                    }
                }
            }

            mod some {
                mod single {
                    use bauer::Builder;

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct Skip {
                        other: u32,
                        #[builder(skip)]
                        field_a: u32,
                    }

                    #[test]
                    fn default() {
                        let f: Skip = Skip::builder()
                            .other(64)
                            .build()
                            $(.$unwrap())?;

                        assert_eq!(f.other, 64);
                        assert_eq!(f.field_a, 0);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipValue {
                        other: u32,
                        #[builder(skip = 42)]
                        field_a: u32,
                    }

                    #[test]
                    fn value() {
                        let f: SkipValue = SkipValue::builder()
                            .other(64)
                            .build()
                            $(.$unwrap())?;

                        assert_eq!(f.other, 64);
                        assert_eq!(f.field_a, 42);
                    }
                }

                mod multi {
                    use bauer::Builder;

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipValueDefault {
                        other: u32,
                        #[builder(skip = 42)]
                        field_a: u32,
                        #[builder(skip)]
                        field_b: u32,
                    }

                    #[test]
                    fn value_default() {
                        let f: SkipValueDefault = SkipValueDefault::builder()
                            .other(64)
                            .build()
                            $(.$unwrap())?;

                        assert_eq!(f.field_a, 42);
                        assert_eq!(f.field_b, 0);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipValue {
                        other: u32,
                        #[builder(skip = 42)]
                        field_a: u32,
                        #[builder(skip = 24)]
                        field_b: u32,
                    }

                    #[test]
                    fn value() {
                        let f: SkipValue = SkipValue::builder()
                            .other(64)
                            .build()
                            $(.$unwrap())?;

                        assert_eq!(f.field_a, 42);
                        assert_eq!(f.field_b, 24);
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct SkipDefault {
                        other: u32,
                        #[builder(skip)]
                        field_a: u32,
                        #[builder(skip)]
                        field_b: u32,
                    }

                    #[test]
                    fn default() {
                        let f: SkipDefault = SkipDefault::builder()
                            .other(64)
                            .build()
                            $(.$unwrap())?;

                        assert_eq!(f.field_a, 0);
                        assert_eq!(f.field_b, 0);
                    }
                }

                mod referrential {
                    use bauer::Builder;

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct OneField {
                        other: u32,
                        #[builder(skip = other + 1)]
                        field_a: u32,
                    }

                    #[test]
                    fn one_field() {
                        for n in [64, 82, 83] {
                            let f: OneField = OneField::builder()
                                .other(n)
                                .build()
                                $(.$unwrap())?;

                            assert_eq!(f.other, n);
                            assert_eq!(f.field_a, n + 1);
                        }
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct Add {
                        x: u32,
                        y: u32,
                        #[builder(skip = x + y)]
                        sum: u32,
                    }

                    #[test]
                    fn two_fields_add() {
                        for x in [89, 123, 450] {
                            for y in [12, 50, 173] {
                                let f: Add = Add::builder()
                                    .x(x)
                                    .y(y)
                                    .build()
                                    $(.$unwrap())?;

                                assert_eq!(f.x, x);
                                assert_eq!(f.y, y);
                                assert_eq!(f.sum, x + y);
                            }
                        }
                    }

                    fn sum(iter: impl Iterator<Item = u32>) -> u32 {
                        iter.sum()
                    }

                    #[derive(Debug, Builder, PartialEq)]
                    #[builder(kind = $kind)]
                    struct CollectedRepeatField {
                        #[builder(repeat = u32, collector = sum)]
                        sum: u32,
                        #[builder(skip = sum * sum)]
                        sum_squared: u32,
                    }

                    #[test]
                    fn collected_repeat_field() {
                        for x in [89, 123] {
                            for y in [12, 50] {
                                for z in [123, 8123] {
                                    let f: CollectedRepeatField = CollectedRepeatField::builder()
                                        .sum(x)
                                        .sum(y)
                                        .sum(z)
                                        .build();

                                    let sum = x + y + z;

                                    assert_eq!(f.sum, sum);
                                    assert_eq!(f.sum_squared, sum * sum);
                                }
                            }
                        }
                    }

                }
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
