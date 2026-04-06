use bauer::Builder;

#[derive(Debug, Builder)]
#[builder(kind = "type-state", prefix = "set_")]
pub struct Foo {
    #[builder(default = "42")]
    pub field_a: u32,
    pub field_b: bool,
    // #[builder(into)]
    // pub field_c: String,
    #[builder(skip_prefix, skip_suffix, rename = "add_d", repeat = u8, repeat_n = 3, collector = field_d_collector)]
    pub field_d: String,
}

fn field_d_collector(iter: impl Iterator<Item = u8>) -> String {
    // [iter[0] + 2, iter[1] + 2, iter[2] + 2]
    iter.map(char::from).collect()
}

fn main() {
    let x: Foo = Foo::builder()
        .set_field_a(69)
        .set_field_b(true)
        // .set_field_c("hello world")
        // .add_d(std::f64::consts::PI.to_string())
        // .add_d(std::f64::consts::TAU.to_string())
        // .add_d(2.72.to_string())
        .add_d(3)
        .add_d(4)
        .add_d(5)
        .build();

    dbg!(x);
}
