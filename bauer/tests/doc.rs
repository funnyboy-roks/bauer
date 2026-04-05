macro_rules! do_thing {
    ($name: ident => $($inner: tt)*) => {
        mod $name {
            use bauer::Builder;

            #[derive(Builder)]
            #[builder(
                doc $($inner)*,
                build_fn_doc $($inner)*,
            )]
            pub struct Struct {
                #[builder(
                    doc(
                        /// Some documentation
                    )
                )]
                field: u8,
            }

            test!();
        }
    }
}

#[rustfmt::skip] // rustfmt really does not like this style of attributes in macro
macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        pub mod $module {
            macro_rules! test {
                () => {
                    #[test]
                    fn build() {
                        let x = Struct::builder().field(0).build().unwrap();
                        assert_eq!(x.field, 0);
                    }
                }
            }

            do_thing!(paren_comment => (
                /// Some documentation
                /// with multiple lines
            ));

            do_thing!(brace_comment => {
                /// Some documentation
                /// with multiple lines
            });

            do_thing!(paren_attribute => (hidden));
            do_thing!(brace_attribute => {hidden});

            do_thing!(paren_comment_attribute => (
                hidden
                /// Some documentation
            ));
            do_thing!(brace_comment_attribute => {
                hidden
                /// Some documentation
            });

            do_thing!(paren_comment_attribute_comma => (
                hidden,
                /// Some documentation
            ));
            do_thing!(brace_comment_attribute_comma => {
                hidden,
                /// Some documentation
            });
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
