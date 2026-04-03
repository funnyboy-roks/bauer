// Adapted from https://github.com/welf/type-state-builder/blob/main/tests/test_raw_identifiers.rs

//! Test raw identifiers with keywords

macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        mod $module {

            use bauer::Builder;

            #[derive(Builder)]
            #[builder(kind = $kind)]
            struct TestWithKeywords {
                r#type: String,
                r#async: Option<bool>,
            }

            #[test]
            fn test_raw_identifiers() {
                let instance = TestWithKeywords::builder()
                    .r#type("test".to_string())
                    .r#async(true)
                    .build()
                    $(.$unwrap())?;

                assert_eq!(instance.r#type, "test");
                assert_eq!(instance.r#async, Some(true));
            }
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
