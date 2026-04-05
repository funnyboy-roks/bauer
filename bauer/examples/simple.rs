use bauer::Builder;

#[derive(Debug, Builder)]
#[builder(
    kind = "owned",
    prefix = "set_",
    build_fn_attributes(
        /// This is a documentation comment :)
        ///
        /// With multiple lines!
    ),
    attributes(
        /// This is a documentation comment :)
        ///
        /// With multiple lines!
    )
)]
pub struct Foo {
    #[builder(default = "42", attributes())]
    pub field_a: u32,
    pub field_b: bool,
    #[builder(into)]
    pub field_c: String,
    #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat)]
    pub field_d: [String; 3],
}

fn main() {
    let x: Foo = Foo::builder()
        .set_field_a(69)
        .set_field_b(true)
        .set_field_c("hello world")
        .add_d(std::f64::consts::PI.to_string())
        .add_d(std::f64::consts::TAU.to_string())
        .add_d(2.72.to_string())
        .build()
        .unwrap();

    dbg!(x);
}
