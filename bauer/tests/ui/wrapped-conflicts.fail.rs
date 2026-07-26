#[derive(bauer::Builder)]
struct UnsupportedRequired {
    #[builder(required)] // invalid
    a: u8,
    #[builder(required)] // invalid
    b: Vec<u8>,
    #[builder(required)] // invalid
    c: (u8, u8),
    #[builder(required)] // invalid
    d: [u8; 3],
    #[builder(required)] // okay
    e: Option<u8>,
}

#[derive(bauer::Builder)]
struct UnsupportedRequiredArgs {
    #[builder(repeat, required)] // invalid
    q: Vec<u8>,
    #[builder(repeat, required)] // invalid
    b: Vec<Option<u8>>,
    #[builder(required, repeat)] // okay (because from_iter on Option)
    c: Option<Vec<u8>>,
}

#[derive(bauer::Builder)]
struct Options {
    #[builder(required)] // okay
    a: Option<u8>,
    #[builder(required)] // invalid (we only check for Option<T>)
    f: std::option::Option<u8>,
}

fn main() {}
