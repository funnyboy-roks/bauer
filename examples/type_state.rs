use bauer::Builder;

#[derive(Debug, Builder)]
#[builder(kind = "type-state")]
struct Foo<T> {
    field_a: String,
    field_b: u32,
    field_c: T,
}

// #[derive(Debug)]
// struct Foo {
//     field_a: String,
//     field_b: u32,
// }
//
// #[non_exhaustive]
// struct FieldAUnset;
// #[non_exhaustive]
// struct FieldASet;
//
// #[non_exhaustive]
// struct FieldBUnset;
// #[non_exhaustive]
// struct FieldBSet;
//
// pub struct FooBuilder<FieldA, FieldB> {
//     field_a: Option<String>,
//     field_b: Option<u32>,
//     _state: PhantomData<(FieldA, FieldB)>,
// }
//
// impl FooBuilder<FieldAUnset, FieldBUnset> {
//     pub fn new() -> Self {
//         Self {
//             field_a: None,
//             field_b: None,
//             _state: PhantomData,
//         }
//     }
// }
//
// impl<FieldB> FooBuilder<FieldAUnset, FieldB> {
//     pub fn field_a(self, field_a: String) -> FooBuilder<FieldASet, FieldB> {
//         FooBuilder {
//             field_a: Some(field_a),
//             field_b: self.field_b,
//             _state: PhantomData,
//         }
//     }
// }
//
// impl<FieldA> FooBuilder<FieldA, FieldBUnset> {
//     pub fn field_b(self, field_b: u32) -> FooBuilder<FieldA, FieldBSet> {
//         FooBuilder {
//             field_a: self.field_a,
//             field_b: Some(field_b),
//             _state: PhantomData,
//         }
//     }
// }
//
// impl FooBuilder<FieldASet, FieldBSet> {
//     pub fn build(self) -> Foo {
//         Foo {
//             field_a: self.field_a.unwrap(),
//             field_b: self.field_b.unwrap(),
//         }
//     }
// }

fn main() {
    let foo = FooBuilder::new()
        .field_a("hello".into())
        .field_b(69)
        .field_c(69)
        .build();
    // let foo = FooBuilder::new() /*.field_a("hello world".into())*/
    //     .build();
    dbg!(foo);
}
