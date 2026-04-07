macro_rules! do_thing {
    ($name: ident, $kind: literal $($unwrap: ident)?, $build_fn_name: ident => $($inner: tt)*) => {
        mod $name {

            use bauer::Builder;

            #[derive(Builder)]
            #[builder(
                kind = $kind,
                build_fn $($inner)*
            )]
            pub struct Struct {
                field: u8,
            }

            #[test]
            fn build() {
                let x = Struct::builder()
                    .field(0)
                    .$build_fn_name()
                    $(.$unwrap())?;
                assert_eq!(x.field, 0);
            }
        }
    }
}

#[rustfmt::skip] // rustfmt really does not like this style of attributes in macro
macro_rules! tests {
    ($kind: literal in mod $module: ident $($unwrap: ident)?) => {
        pub mod $module {
            do_thing!(paren_empty, $kind $($unwrap)?, build => ());
            do_thing!(brace_empty, $kind $($unwrap)?, build => {});

            do_thing!(paren_attribute, $kind $($unwrap)?, build => (
                attribute(#[attribute::my_attribute]),
                attributes(#[attribute::my_attribute]),
            ));
            do_thing!(brace_attribute, $kind $($unwrap)?, build => {
                attribute(#[attribute::my_attribute]),
                attributes(#[attribute::my_attribute]),
            });

            do_thing!(paren_doc_hidden, $kind $($unwrap)?, build => (
                doc(hidden),
                docs(hidden),
            ));
            do_thing!(brace_doc_hidden, $kind $($unwrap)?, build => {
                doc(hidden),
                docs(hidden),
            });

            do_thing!(paren_doc, $kind $($unwrap)?, build => (
                doc(
                    /// Some docs
                ),
                docs(
                    /// Some docs
                ),
            ));
            do_thing!(brace_doc, $kind $($unwrap)?, build => {
                doc(
                    /// Some docs
                ),
                docs(
                    /// Some docs
                ),
            });

            do_thing!(paren_rename1, $kind $($unwrap)?, finish => (
                rename = "finish"
            ));
            do_thing!(brace_rename1, $kind $($unwrap)?, finish => {
                rename = "finish"
            });

            do_thing!(paren_rename2, $kind $($unwrap)?, complete => (
                rename = "complete"
            ));
            do_thing!(brace_rename2, $kind $($unwrap)?, complete => {
                rename = "complete"
            });
        }
    };
}

tests!("borrowed" in mod borrowed unwrap);
tests!("owned" in mod owned unwrap);
tests!("type-state" in mod type_state);
