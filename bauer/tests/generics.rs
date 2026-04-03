use bauer::Builder;

#[test]
fn all_generics() {
    macro_rules! define {
        ($name: ident, $kind: literal) => {
            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct $name<'a, const N: usize, T: ?Sized> {
                field: [&'a T; N],
            }
        };
    }

    macro_rules! populate {
        ($name: ident) => {
            $name::builder().field(["hello", "world"])
        };
    }

    macro_rules! expected {
        ($name: ident) => {
            $name {
                field: ["hello", "world"],
            }
        };
    }

    define!(BorrowedFoo, "borrowed");
    let foo: BorrowedFoo<2, str> = populate!(BorrowedFoo).build().unwrap();
    assert_eq!(foo, expected!(BorrowedFoo));

    define!(OwnedFoo, "owned");
    let foo: OwnedFoo<2, str> = populate!(OwnedFoo).build().unwrap();
    assert_eq!(foo, expected!(OwnedFoo));

    define!(TypeStateFoo, "type-state");
    let foo: TypeStateFoo<2, str> = populate!(TypeStateFoo).build();
    assert_eq!(foo, expected!(TypeStateFoo));
}

#[test]
fn type_generics() {
    macro_rules! define {
        ($name: ident, $kind: literal) => {
            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct $name<A, B, C> {
                field: (A, B, C),
            }
        };
    }

    macro_rules! populate {
        ($name: ident) => {
            $name::builder().field((0, "hello", 'c'))
        };
    }

    macro_rules! expected {
        ($name: ident) => {
            $name {
                field: (0, "hello", 'c'),
            }
        };
    }

    define!(BorrowedFoo, "borrowed");
    let foo: BorrowedFoo<i32, &str, char> = populate!(BorrowedFoo).build().unwrap();
    assert_eq!(foo, expected!(BorrowedFoo));

    define!(OwnedFoo, "owned");
    let foo: OwnedFoo<i32, &str, char> = populate!(OwnedFoo).build().unwrap();
    assert_eq!(foo, expected!(OwnedFoo));

    define!(TypeStateFoo, "type-state");
    let foo: TypeStateFoo<i32, &str, char> = populate!(TypeStateFoo).build();
    assert_eq!(foo, expected!(TypeStateFoo));
}

#[test]
fn const_generics() {
    macro_rules! define {
        ($name: ident, $kind: literal) => {
            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct $name<const A: usize, const B: char, const C: bool> {
                field: [u8; A],
            }
        };
    }

    macro_rules! populate {
        ($name: ident) => {
            $name::builder().field([0, 1, 2, 3])
        };
    }

    macro_rules! expected {
        ($name: ident) => {
            $name {
                field: [0, 1, 2, 3],
            }
        };
    }

    define!(BorrowedFoo, "borrowed");
    let foo: BorrowedFoo<4, 'c', true> = populate!(BorrowedFoo).build().unwrap();
    assert_eq!(foo, expected!(BorrowedFoo));

    define!(OwnedFoo, "owned");
    let foo: OwnedFoo<4, 'c', true> = populate!(OwnedFoo).build().unwrap();
    assert_eq!(foo, expected!(OwnedFoo));

    define!(TypeStateFoo, "type-state");
    let foo: TypeStateFoo<4, 'c', true> = populate!(TypeStateFoo).build();
    assert_eq!(foo, expected!(TypeStateFoo));
}

#[test]
fn lifetimes() {
    let a_string = String::from("hello");
    let a = &a_string;
    let b: &'static str = "world";
    macro_rules! define {
        ($name: ident, $kind: literal) => {
            #[derive(Debug, Builder, PartialEq)]
            #[builder(kind = $kind)]
            struct $name<'a, 'b> {
                short: &'a str,
                statik: &'b str,
            }
        };
    }

    macro_rules! populate {
        ($name: ident) => {
            $name::builder().short(a).statik(b)
        };
    }

    macro_rules! expected {
        ($name: ident) => {
            $name {
                short: a,
                statik: b,
            }
        };
    }

    define!(BorrowedFoo, "borrowed");
    let foo: BorrowedFoo<'_, 'static> = populate!(BorrowedFoo).build().unwrap();
    assert_eq!(foo, expected!(BorrowedFoo));

    define!(OwnedFoo, "owned");
    let foo: OwnedFoo<'_, 'static> = populate!(OwnedFoo).build().unwrap();
    assert_eq!(foo, expected!(OwnedFoo));

    define!(TypeStateFoo, "type-state");
    let foo: TypeStateFoo<'_, 'static> = populate!(TypeStateFoo).build();
    assert_eq!(foo, expected!(TypeStateFoo));

    drop(a_string);
}
