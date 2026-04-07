macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {
            use bauer::Builder;

            #[derive(Debug, Builder)]
            #[builder(kind = $kind)]
            struct All<'a, const N: usize, T: ?Sized> {
                field: [&'a T; N],
            }

            #[test]
            fn multi_generics() {
                let all: All<2, str> = All::builder()
                    .field(["hello", "world"])
                    .build()
                    $(.$unwrap())?;

                assert_eq!(all.field, ["hello", "world"]);
            }

            mod type_generics {
                use bauer::Builder;

                #[derive(Debug, Builder)]
                #[builder(kind = $kind)]
                struct Type<A, B, C> {
                    field: (A, B, C),
                }

                #[test]
                fn simple() {
                    let ty: Type<i32, &str, char> = Type::builder()
                        .field((0, "hello", 'c'))
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(ty.field, (0, "hello", 'c'));
                }

                #[derive(Debug, Builder)]
                #[builder(kind = $kind)]
                struct TypeTuple<A, B, C> {
                    #[builder(tuple)]
                    field: (A, B, C),
                }

                #[test]
                fn tuple() {
                    let ty: TypeTuple<i32, &str, char> = TypeTuple::builder()
                        .field(0, "hello", 'c')
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(ty.field, (0, "hello", 'c'));
                }


                #[derive(Debug, Builder)]
                #[builder(kind = $kind)]
                struct TypeInto<A, B, C> {
                    #[builder(tuple, into)]
                    field: (A, B, C),
                }

                #[test]
                fn into() {
                    let ty: TypeInto<std::path::PathBuf, String, char> = TypeInto::builder()
                        .field("/bin/foo", "hello", b'c')
                        .build()
                        $(.$unwrap())?;

                    assert_eq!(ty.field.0, std::path::Path::new("/bin/foo"));
                    assert_eq!(ty.field.1, "hello");
                    assert_eq!(ty.field.2, 'c');
                }
            }

            #[derive(Debug, Builder)]
            #[builder(kind = $kind)]
            struct Const<const A: usize, const B: char, const C: bool> {
                field: [u8; A],
            }

            #[test]
            fn const_generics() {
                let c: Const<4, 'c', true> = Const::builder()
                    .field([0, 1, 2, 3])
                    .build()
                    $(.$unwrap())?;
                assert_eq!(c.field, [0, 1, 2, 3]);
            }


            #[derive(Debug, Builder)]
            #[builder(kind = $kind)]
            struct Lifetimes<'a, 'b> {
                short: &'a str,
                statik: &'b str,
            }

            #[test]
            fn lifetimes() {
                let a_string = String::from("hello");
                let a = &a_string;
                let b: &'static str = "world";

                let lt: Lifetimes<'_, 'static> = Lifetimes::builder()
                    .short(a)
                    .statik(b)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(lt.short, a);
                assert_eq!(lt.statik, b);

                drop(a_string);
            }

            // adapted from https://github.com/welf/type-state-builder/blob/main/tests/associated_type_test.rs
            trait Trait {
                type Default: Default;
                type From: From<u32>;
                type Repeat: FromIterator<char>;
            }

            #[derive(Debug)]
            struct Inner;
            impl Trait for Inner {
                type Default = String;
                type From = u64;
                type Repeat = Vec<char>;
            }

            #[derive(Debug, Builder)]
            #[builder(kind = $kind)]
            struct AssocType<T: Trait> {
                #[builder(default)]
                default: T::Default,
                #[builder(into)]
                from: T::From,
                #[builder(repeat = char)]
                repeat: T::Repeat,
            }

            #[test]
            fn associated_types() {
                let at: AssocType<Inner> = AssocType::<Inner>::builder()
                    .default(String::from("hello"))
                    .from(3u32)
                    .repeat('h')
                    .repeat('i')
                    .build()
                    $(.$unwrap())?;

                assert_eq!(at.default, "hello");
                assert_eq!(at.from, 3u64);
                assert_eq!(at.repeat, ['h', 'i']);
            }

            #[test]
            fn associated_types_default() {
                let at: AssocType<Inner> = AssocType::<Inner>::builder()
                    .from(3u32)
                    .repeat('h')
                    .repeat('i')
                    .build()
                    $(.$unwrap())?;

                assert_eq!(at.default, "");
                assert_eq!(at.from, 3u64);
                assert_eq!(at.repeat, ['h', 'i']);
            }


        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
