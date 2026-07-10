use bauer::Builder;

#[derive(Builder)]
#[builder(build_fn(
    map = |x| 0 // no return type
))]
pub struct Foo {}

#[derive(Builder)]
#[builder(build_fn(
    map = |x, y| -> u32 { 0 } // multiple arguments
))]
pub struct Foo2 {}

#[derive(Builder)]
#[builder(build_fn(
    map = || -> u32 { 0 } // no arguments
))]
pub struct Foo3 {}

fn main() {}
