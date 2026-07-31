use bauer::Builder;

#[derive(Builder)]
struct Foo {
    #[builder(flag)] // fail
    a: Vec<u32>,
    #[builder(flag)] // fail
    b: std::primitive::bool,
    #[builder(default, flag)] // fail
    c: bool,
    #[builder(flag, default)] // fail
    d: bool,
}

fn main() {}
