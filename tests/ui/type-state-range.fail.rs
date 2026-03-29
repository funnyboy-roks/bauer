use bauer::Builder;

#[derive(Builder)]
#[builder(kind = "type-state")]
struct RangeInclusive {
    #[builder(repeat, repeat_n = 2..=3)]
    field_a: Vec<u32>,
}

#[derive(Builder)]
#[builder(kind = "type-state")]
struct RangeExclusive {
    #[builder(repeat, repeat_n = 2..3)]
    field_a: Vec<u32>,
}

#[derive(Builder)]
#[builder(kind = "type-state")]
struct RangeOpen {
    #[builder(repeat, repeat_n = 2..)]
    field_a: Vec<u32>,
}

#[derive(Builder)]
#[builder(kind = "type-state")]
struct Exact {
    #[builder(repeat, repeat_n = 2)]
    field_a: Vec<u32>,
}

fn main() {
    let _: RangeInclusive = RangeInclusive::builder().field_a(1).build(); // fail b/c <2
    let _: RangeInclusive = RangeInclusive::builder()
        .field_a(1)
        .field_a(2)
        .field_a(3)
        .field_a(4)
        .build(); // fail b/c >3

    let _: RangeExclusive = RangeExclusive::builder().field_a(1).build(); // fail b/c <2
    let _: RangeExclusive = RangeExclusive::builder()
        .field_a(1)
        .field_a(2)
        .field_a(3)
        .build(); // fail b/c >=3

    let _: RangeOpen = RangeOpen::builder().field_a(1).build(); // fail b/c <2

    let _: Exact = Exact::builder().field_a(1).build(); // fail b/c <2
    let _: Exact = Exact::builder().field_a(1).field_a(2).field_a(2).build(); // fail b/c >2
}
