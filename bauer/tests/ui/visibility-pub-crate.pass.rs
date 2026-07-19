mod foo {
    pub mod bar {
        #[derive(bauer::Builder)]
        pub(crate) struct PubCrateStruct {
            #[builder(default)]
            foo: u32,
        }

        fn next_to_it() {
            PubCrateStruct::builder().foo(32).build(); // compiles
        }
    }

    fn in_super() {
        bar::PubCrateStruct::builder().foo(32).build(); // compiles
    }
}

mod baz {
    fn in_crate() {
        crate::foo::bar::PubCrateStruct::builder().foo(32).build(); // compiles
    }
}

fn main() {}
