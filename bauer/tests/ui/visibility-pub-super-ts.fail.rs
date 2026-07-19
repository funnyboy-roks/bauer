mod foo {
    pub mod bar {
        #[derive(bauer::Builder)]
        #[builder(kind = "type-state")]
        pub(super) struct PubSuperStruct {
            #[builder(default)]
            foo: u32,
        }

        #[derive(bauer::Builder)]
        #[builder(visibility = pub(super))]
        #[builder(kind = "type-state")]
        pub struct PubSuperAttr {
            #[builder(default)]
            foo: u32,
        }

        fn next_to_it() {
            PubSuperStruct::builder().foo(32).build(); // compiles
            PubSuperAttr::builder().foo(32).build(); // compiles
        }
    }

    fn in_super() {
        bar::PubSuperStruct::builder().foo(32).build(); // compiles
        bar::PubSuperAttr::builder().foo(32).build(); // compiles
    }
}

mod baz {
    fn in_crate() {
        crate::foo::bar::PubSuperStruct::builder().foo(32).build(); // fails to compile
        crate::foo::bar::PubSuperAttr::builder().foo(32).build(); // fails to compile (on builder/foo)
    }
}

fn main() {}
