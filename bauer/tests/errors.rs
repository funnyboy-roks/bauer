macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            mod missing {
                use bauer::Builder;

                #[derive(Debug, Builder)]
                struct RequiredFields {
                    field_a: u32,
                }

                #[test]
                fn missing_field_err() {
                    let err = RequiredFields::builder().build().unwrap_err();
                    assert_eq!(err, RequiredFieldsBuildError::MissingFieldA);
                }

                #[test]
                fn missing_none_ok() {
                    let ok = RequiredFields::builder().field_a(42).build().unwrap();
                    assert_eq!(ok.field_a, 42);
                }
            }

            mod missing_default {
                use bauer::Builder;

                #[derive(Debug, Builder)]
                struct RequiredFields {
                    #[builder(default)]
                    field_a: u32,
                    field_b: u32,
                }

                #[test]
                fn missing_default_ok() {
                    let ok = RequiredFields::builder().field_b(42).build().unwrap();
                    assert_eq!(ok.field_a, 0);
                    assert_eq!(ok.field_b, 42);
                }

                #[test]
                fn missing_required_err() {
                    let err = RequiredFields::builder().field_a(42).build().unwrap_err();
                    assert_eq!(err, RequiredFieldsBuildError::MissingFieldB);
                }

                #[test]
                fn missing_both_err() {
                    let err = RequiredFields::builder().build().unwrap_err();
                    assert_eq!(err, RequiredFieldsBuildError::MissingFieldB);
                }
            }

            mod array {
                use bauer::Builder;
                #[derive(Debug, Builder)]
                struct Repeat {
                    #[builder(repeat)]
                    exact: [u32; 3],
                }

                #[test]
                fn absent_err() {
                    let err = Repeat::builder().build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeExact(0));
                }

                #[test]
                fn less_err() {
                    let err = Repeat::builder().exact(1).build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeExact(1));
                }

                #[test]
                fn more_err() {
                    let err = Repeat::builder()
                        .exact(1)
                        .exact(2)
                        .exact(3)
                        .exact(4)
                        .build()
                        .unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeExact(4));
                }

                #[test]
                fn equal_ok() {
                    let ok = Repeat::builder()
                        .exact(1)
                        .exact(2)
                        .exact(3)
                        .build()
                        .unwrap();
                    assert_eq!(ok.exact, [1, 2, 3]);
                }
            }

            mod exact {
                use bauer::Builder;
                #[derive(Debug, Builder)]
                struct Repeat {
                    #[builder(repeat, repeat_n = 3)]
                    exact: Vec<u32>,
                }

                #[test]
                fn absent_err() {
                    let err = Repeat::builder().build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeExact(0));
                }

                #[test]
                fn less_err() {
                    let err = Repeat::builder().exact(1).build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeExact(1));
                }

                #[test]
                fn more_err() {
                    let err = Repeat::builder()
                        .exact(1)
                        .exact(2)
                        .exact(3)
                        .exact(4)
                        .build()
                        .unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeExact(4));
                }

                #[test]
                fn equal_ok() {
                    let ok = Repeat::builder()
                        .exact(1)
                        .exact(2)
                        .exact(3)
                        .build()
                        .unwrap();
                    assert_eq!(ok.exact, [1, 2, 3]);
                }
            }

            mod min {
                use bauer::Builder;

                #[derive(Debug, Builder)]
                struct Repeat {
                    #[builder(repeat, repeat_n = 3..)]
                    min: Vec<u32>,
                }

                #[test]
                fn absent_err() {
                    let err = Repeat::builder().build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeMin(0));
                }

                #[test]
                fn less_err() {
                    let err = Repeat::builder().min(1).build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeMin(1));
                }

                #[test]
                fn more_ok() {
                    let ok = Repeat::builder()
                        .min(1)
                        .min(2)
                        .min(3)
                        .min(4)
                        .build()
                        .unwrap();
                    assert_eq!(ok.min, [1, 2, 3, 4]);
                }

                #[test]
                fn equal_ok() {
                    let ok = Repeat::builder().min(1).min(2).min(3).build().unwrap();
                    assert_eq!(ok.min, [1, 2, 3]);
                }
            }

            mod max {
                use bauer::Builder;

                #[derive(Debug, Builder)]
                struct Repeat {
                    #[builder(repeat, repeat_n = ..3)]
                    max: Vec<u32>,
                }

                #[test]
                fn absent_ok() {
                    let ok = Repeat::builder().build().unwrap();
                    assert_eq!(ok.max, []);
                }

                #[test]
                fn less_ok() {
                    let ok = Repeat::builder().max(1).build().unwrap();
                    assert_eq!(ok.max, [1]);
                }

                #[test]
                fn more_err() {
                    let err = Repeat::builder()
                        .max(1)
                        .max(2)
                        .max(3)
                        .max(4)
                        .build()
                        .unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeMax(4));
                }

                #[test]
                fn equal_err() {
                    let err = Repeat::builder().max(1).max(2).max(3).build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeMax(3));
                }
            }

            mod closed {
                use bauer::Builder;

                #[derive(Debug, Builder)]
                struct Repeat {
                    #[builder(repeat, repeat_n = 2..3)]
                    closed: Vec<u32>,
                }

                #[test]
                fn absent_err() {
                    let err = Repeat::builder().build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeClosed(0));
                }

                #[test]
                fn less_err() {
                    let err = Repeat::builder().closed(1).build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeClosed(1));
                }

                #[test]
                fn more_err() {
                    let err = Repeat::builder()
                        .closed(1)
                        .closed(2)
                        .closed(3)
                        .closed(4)
                        .build()
                        .unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeClosed(4));
                }

                #[test]
                fn equal_min_ok() {
                    let ok = Repeat::builder().closed(1).closed(2).build().unwrap();
                    assert_eq!(ok.closed, [1, 2]);
                }

                #[test]
                fn equal_max_err() {
                    let err = Repeat::builder()
                        .closed(1)
                        .closed(2)
                        .closed(3)
                        .build()
                        .unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeClosed(3));
                }
            }

            mod closed_eq {
                use bauer::Builder;

                #[derive(Debug, Builder)]
                struct Repeat {
                    #[builder(repeat, repeat_n = 2..=3)]
                    closed: Vec<u32>,
                }

                #[test]
                fn absent_err() {
                    let err = Repeat::builder().build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeClosed(0));
                }

                #[test]
                fn less_err() {
                    let err = Repeat::builder().closed(1).build().unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeClosed(1));
                }

                #[test]
                fn more_err() {
                    let err = Repeat::builder()
                        .closed(1)
                        .closed(2)
                        .closed(3)
                        .closed(4)
                        .build()
                        .unwrap_err();
                    assert_eq!(err, RepeatBuildError::RangeClosed(4));
                }

                #[test]
                fn equal_min_ok() {
                    let ok = Repeat::builder().closed(1).closed(2).build().unwrap();
                    assert_eq!(ok.closed, [1, 2]);
                }

                #[test]
                fn equal_max_ok() {
                    let ok = Repeat::builder()
                        .closed(1)
                        .closed(2)
                        .closed(3)
                        .build()
                        .unwrap();
                    assert_eq!(ok.closed, [1, 2, 3]);
                }
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
// Not using type-state since it doesn't have runtime errors
