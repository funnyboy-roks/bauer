mod foo {
    pub mod bar {
        #[derive(bauer::Builder)]
        #[builder(kind = "type-state")]
        struct NoVis {
            #[builder(default)]
            foo: u32,
        }

        fn next_to_it() {
            NoVis::builder().foo(32).build(); // compiles
        }
    }

    fn in_super() {
        bar::NoVis::builder().foo(32).build(); // fails to compile
    }
}

mod baz {
    fn in_crate() {
        crate::foo::bar::NoVis::builder().foo(32).build(); // fails to compile
    }
}

fn main() {}
