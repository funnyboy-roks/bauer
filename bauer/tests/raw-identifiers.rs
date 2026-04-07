// Adapted from https://github.com/welf/type-state-builder/blob/main/tests/test_raw_identifiers.rs

//! Test raw identifiers with keywords

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {

            #[test]
            fn fields() {
                use bauer::Builder;

                #[derive(Builder)]
                #[builder(kind = $kind)]
                struct TestWithKeywords {
                    r#type: String,
                    r#async: Option<bool>,
                }

                let instance = TestWithKeywords::builder()
                    .r#type("test".to_string())
                    .r#async(true)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(instance.r#type, "test");
                assert_eq!(instance.r#async, Some(true));
            }

            #[test]
            fn generics() {
                #![allow(non_camel_case_types)]

                use bauer::Builder;

                #[derive(Builder)]
                #[builder(kind = $kind)]
                struct TestWithKeywords<r#type> {
                    field: r#type,
                }

                let instance = TestWithKeywords::builder()
                    .field("test".to_string())
                    .build()
                    $(.$unwrap())?;

                assert_eq!(instance.field, "test");
            }

            #[test]
            fn struct_name() {
                #![allow(non_camel_case_types)]

                use bauer::Builder;

                #[derive(Builder)]
                #[builder(kind = $kind)]
                struct r#type {
                    field: String,
                }

                let instance = r#type::builder()
                    .field("test".to_string())
                    .build()
                    $(.$unwrap())?;

                assert_eq!(instance.field, "test");
            }

            #[test]
            fn rename() {
                use bauer::Builder;

                #[derive(Builder)]
                #[builder(kind = $kind)]
                struct Foo {
                    #[builder(rename = "r#type")]
                    field: String,
                }

                let instance = Foo::builder()
                    .r#type("test".to_string())
                    .build()
                    $(.$unwrap())?;

                assert_eq!(instance.field, "test");
            }

            #[test]
            fn prefix() {
                use bauer::Builder;

                #[derive(Builder)]
                #[builder(kind = $kind, prefix = "ty")]
                struct Foo {
                    pe: String,
                }

                let instance = Foo::builder()
                    .r#type("test".to_string())
                    .build()
                    $(.$unwrap())?;

                assert_eq!(instance.pe, "test");
            }

            #[test]
            fn suffix() {
                use bauer::Builder;

                #[derive(Builder)]
                #[builder(kind = $kind, suffix = "pe")]
                struct Foo {
                    ty: String,
                }

                let instance = Foo::builder()
                    .r#type("test".to_string())
                    .build()
                    $(.$unwrap())?;

                assert_eq!(instance.ty, "test");
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
