#![allow(unused)]

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {

            macro_rules! suite {
                (mod $inner_mod: ident => $iter: ident) => {
                    mod $inner_mod {
                        use bauer::Builder;

                        fn field_collector(iter: impl $iter<Item = u32>) -> u32 {
                            iter.sum()
                        }

                        #[derive(Builder)]
                        #[builder(kind = $kind)]
                        struct Foo {
                            #[builder(repeat = u32, collector = field_collector)]
                            field: u32,
                        }

                        #[test]
                        fn sum() {
                            let foo: Foo = Foo::builder().field(1).field(2).field(3).build();
                            assert_eq!(foo.field, 1 + 2 + 3);
                        }


                        fn field_counter(iter: impl $iter<Item = u32>) -> usize {
                            iter.count()
                        }

                        #[derive(Builder)]
                        #[builder(kind = $kind)]
                        struct Count {
                            #[builder(repeat = u32, collector = field_counter)]
                            field: usize,
                        }

                        #[test]
                        fn count() {
                            let foo: Count = Count::builder().field(1).field(2).field(3).build();
                            assert_eq!(foo.field, 3);
                        }


                        #[derive(Builder)]
                        #[builder(kind = $kind)]
                        struct UpperBound {
                            #[builder(repeat = u32, repeat_n = ..5, collector = field_counter)]
                            field: usize,
                        }

                        #[test]
                        fn count_upper_bound() {
                            let foo: UpperBound = UpperBound::builder()
                                .field(1)
                                .field(2)
                                .field(3)
                                .build()
                                $(.$unwrap())?;
                            assert_eq!(foo.field, 3);
                        }


                        #[derive(Builder)]
                        #[builder(kind = $kind)]
                        struct Exact {
                            #[builder(repeat = u32, repeat_n = 5, collector = field_counter)]
                            field: usize,
                        }

                        #[test]
                        fn count_exact() {
                            let foo: Exact = Exact::builder()
                                .field(1)
                                .field(2)
                                .field(3)
                                .field(4)
                                .field(5)
                                .build()
                                $(.$unwrap())?;
                            assert_eq!(foo.field, 5);
                        }



                        #[derive(Builder)]
                        #[builder(kind = $kind)]
                        struct LowerBound {
                            #[builder(repeat = u32, repeat_n = 2.., collector = field_counter)]
                            field: usize,
                        }

                        #[test]
                        fn count_lower_bound() {
                            let foo: LowerBound = LowerBound::builder()
                                .field(1)
                                .field(2)
                                .field(3)
                                .build()
                                $(.$unwrap())?;
                            assert_eq!(foo.field, 3);
                        }



                        #[derive(Builder)]
                        #[builder(kind = $kind)]
                        struct ClosedRange {
                            #[builder(repeat = u32, repeat_n = 2..5, collector = field_counter)]
                            field: usize,
                        }

                        #[test]
                        fn count_closed_range() {
                            let foo: ClosedRange = ClosedRange::builder()
                                .field(1)
                                .field(2)
                                .field(3)
                                .build()
                                $(.$unwrap())?;
                            assert_eq!(foo.field, 3);
                        }


                        mod generic {
                            mod unbounded {
                                use bauer::Builder;

                                fn generic_collector<T>(iter: impl ExactSizeIterator<Item = T>) -> usize {
                                    iter.len()
                                }

                                #[derive(Builder)]
                                #[builder(kind = $kind)]
                                struct U32Collector {
                                    #[builder(repeat = u32, repeat_n = 2..5, collector = generic_collector)]
                                    field: usize,
                                }

                                #[test]
                                fn unbounded_u32() {
                                    let foo: U32Collector = U32Collector::builder()
                                        .field(1)
                                        .field(2)
                                        .field(3)
                                        .build()
                                        $(.$unwrap())?;

                                    assert_eq!(foo.field, 3);
                                }

                                #[derive(Builder)]
                                #[builder(kind = $kind)]
                                struct StrCollector {
                                    #[builder(repeat = String, into, repeat_n = 2..5, collector = generic_collector)]
                                    field: usize,
                                }

                                #[test]
                                fn unbounded_str() {
                                    let foo: StrCollector = StrCollector::builder()
                                        .field("h")
                                        .field("h")
                                        .field("h")
                                        .build()
                                        $(.$unwrap())?;

                                    assert_eq!(foo.field, 3);
                                }
                            }

                            mod bounded {
                                use bauer::Builder;

                                fn to_string_collector<T: ToString>(iter: impl ExactSizeIterator<Item = T>) -> String {
                                    iter.map(|x| x.to_string()).collect()
                                }

                                #[derive(Builder)]
                                #[builder(kind = $kind)]
                                struct U32Collector {
                                    #[builder(repeat = u32, repeat_n = 2..5, collector = to_string_collector)]
                                    field: String,
                                }

                                #[test]
                                fn u32() {
                                    let foo: U32Collector = U32Collector::builder()
                                        .field(1)
                                        .field(2)
                                        .field(3)
                                        .build()
                                        $(.$unwrap())?;

                                    assert_eq!(foo.field, "123");
                                }

                                #[derive(Builder)]
                                #[builder(kind = $kind)]
                                struct StrCollector {
                                    #[builder(repeat = String, into, repeat_n = 2..5, collector = to_string_collector)]
                                    field: String,
                                }

                                #[test]
                                fn str() {
                                    let foo: StrCollector = StrCollector::builder()
                                        .field("h")
                                        .field("i")
                                        .field("j")
                                        .build()
                                        $(.$unwrap())?;

                                    assert_eq!(foo.field, "hij");
                                }
                            }

                        }

                    }
                };
            }

            suite!(mod iterator => Iterator);
            suite!(mod exact_size_iterator => ExactSizeIterator);
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
