#![allow(dead_code)]

// enusre that the same functionality is generated for all kinds (simple)

use bauer::Builder;

macro_rules! define {
    ($name: ident, $kind: literal) => {
        #[derive(Debug, Builder, PartialEq)]
        #[builder(kind = $kind)]
        struct $name {
            no_repeat: [char; 3],
            #[builder(repeat)]
            repeat: [u32; 3],
            #[builder(repeat, into)]
            repeat_into: [String; 3],
            #[builder(repeat, tuple)]
            repeat_tuple: [(u8, u8); 3],
            #[builder(repeat, adapter = |x: u32| x + 2)]
            repeat_adapter: [u32; 4],
        }
    };
}

macro_rules! populate {
    ($name: ident) => {
        $name::builder()
            .no_repeat(['a', 'b', 'c'])
            .repeat(0)
            .repeat(1)
            .repeat(2)
            .repeat_into("foo")
            .repeat_into("bar")
            .repeat_into("baz")
            .repeat_tuple(1, 6)
            .repeat_tuple(2, 5)
            .repeat_tuple(3, 4)
            .repeat_adapter(6)
            .repeat_adapter(12)
            .repeat_adapter(18)
            .repeat_adapter(24)
    };
}

macro_rules! expected {
    ($name: ident) => {
        $name {
            no_repeat: ['a', 'b', 'c'],
            repeat: [0, 1, 2],
            repeat_into: ["foo".into(), "bar".into(), "baz".into()],
            repeat_tuple: [(1, 6), (2, 5), (3, 4)],
            repeat_adapter: [8, 14, 20, 26],
        }
    };
}

#[test]
fn array_owned() {
    define!(Foo, "owned");
    let x = populate!(Foo).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}

#[test]
fn array_borrowed() {
    define!(Foo, "borrowed");
    let x = populate!(Foo).build().unwrap();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}

#[test]
fn array_type_state() {
    define!(Foo, "type-state");
    let x = populate!(Foo).build();
    dbg!(&x);
    assert_eq!(x, expected!(Foo));
}
