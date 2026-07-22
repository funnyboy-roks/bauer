use bauer::Builder;

#[derive(Builder)]
struct Conflict {
    #[builder(associated, default)] // fail
    field_a: u32,
    #[builder(associated, into)] // okay
    field_b: u32,
    #[builder(associated, repeat)] // fail
    field_c: Vec<u32>,
    #[builder(associated, rename = "foo")] // okay
    field_d: u32,
    #[builder(associated, skip_prefix)] // fail
    field_e: u32,
    #[builder(associated, skip_suffix)] // fail
    field_f: u32,
    #[builder(associated, tuple)] // fail
    field_g: (u32, u32),
    #[builder(associated, adapter = |x: u8| x.into())] // okay
    field_h: u32,
    #[builder(associated, attributes())] // fail
    field_i: u32,
    #[builder(associated, doc())] // fail
    field_j: u32,
    #[builder(associated, collector = FromIterator::from_iter)] // fail
    field_k: u32,
    #[builder(associated, skip)] // fail
    field_l: u32,
    #[builder(associated, visibility = pub)] // fail
    field_m: u32,
}

fn main() {}
