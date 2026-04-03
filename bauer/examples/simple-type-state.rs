use bauer::Builder;

#[derive(Debug, Builder)]
#[builder(kind = "type-state", prefix = "set_")]
pub struct Foo {
    /// Hello
    #[builder(default = "42")]
    pub field_a: u32,
    pub field_b: bool,
    #[builder(into)]
    pub field_c: String,
    #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat, repeat_n = 3..)]
    pub field_d: Vec<f64>,
}

fn main() {
    let builder = Foo::builder()
        .set_field_a(69)
        .set_field_b(true)
        .set_field_c("hello world")
        .add_d(std::f64::consts::PI)
        .add_d(std::f64::consts::TAU)
        .add_d(2.72);

    dbg!(std::any::type_name_of_val(&builder));

    let x: Foo = builder.build();

    dbg!(x);
}
