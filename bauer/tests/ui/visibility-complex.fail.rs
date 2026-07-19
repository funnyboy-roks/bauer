mod foo {
    pub mod bar {
        #[derive(bauer::Builder)]
        #[builder(
            visibility = pub(crate),
            build_fn(
                visibility = pub(self),
            ),
            builder_fn(
                visibility = pub(super),
            ),
        )]
        pub struct Complex {
            #[builder(default)]
            foo: u32,
        }

        fn next_to_it() {
            Complex::builder() // compiles
                .foo(32) // compiles
                .build(); // compiles
        }
    }

    fn in_super() {
        bar::Complex::builder() // compiles
            .foo(32) // compiles
            .build(); // fail
    }
}

mod baz {
    fn in_crate() {
        crate::foo::bar::Complex::builder() // fail
            .foo(32) // compiles
            .build(); // fail
    }
}

fn main() {}
