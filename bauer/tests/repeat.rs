#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
struct Distinct<T>(T);

impl<T> From<T> for Distinct<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Distinct<T> {
    fn into_inner(self) -> T {
        self.0
    }
}

macro_rules! tests {
    ($kind: literal in mod $module: ident) => {
        mod $module {
            use super::Distinct;
            use bauer::Builder;

            #[derive(Builder)]
            struct Repeat {
                #[builder(repeat)]
                repeat: Vec<u32>,
            }

            #[test]
            fn repeat_0() {
                let rep = Repeat::builder().build();
                assert_eq!(rep.repeat, []);
            }

            #[test]
            fn repeat_2() {
                let rep = Repeat::builder().repeat(1).repeat(2).build();
                assert_eq!(rep.repeat, [1, 2]);
            }

            #[derive(Builder)]
            struct RepeatInto {
                #[builder(repeat, into)]
                repeat: Vec<Distinct<u32>>,
            }

            #[test]
            fn repeat_into_0() {
                let rep = RepeatInto::builder().build();
                assert_eq!(rep.repeat, []);
            }

            #[test]
            fn repeat_into_2() {
                let rep = RepeatInto::builder().repeat(1).repeat(2).build();
                assert_eq!(rep.repeat, [Distinct(1u32), Distinct(2u32)]);
            }

            #[derive(Builder)]
            struct RepeatTuple {
                #[builder(repeat, tuple)]
                repeat: Vec<(u32, u32)>,
            }

            #[test]
            fn repeat_tuple_0() {
                let rep = RepeatTuple::builder().build();
                assert_eq!(rep.repeat, []);
            }

            #[test]
            fn repeat_tuple_2() {
                let rep = RepeatTuple::builder().repeat(1, 2).repeat(3, 4).build();
                assert_eq!(rep.repeat, [(1, 2), (3, 4)]);
            }

            use std::collections::{BTreeMap, HashMap};

            #[derive(Builder)]
            struct RepeatHashMap {
                #[builder(repeat = (u32, u32))]
                repeat: HashMap<u32, u32>,
            }

            #[test]
            fn repeat_hashmap_0() {
                let rep = RepeatHashMap::builder().build();
                assert_eq!(rep.repeat, HashMap::new());
            }

            #[test]
            fn repeat_hashmap_2() {
                let rep = RepeatHashMap::builder()
                    .repeat((1, 2))
                    .repeat((3, 4))
                    .build();
                assert_eq!(rep.repeat, HashMap::from([(1, 2), (3, 4)]));
            }

            // just to ensure no hashmap-specific behaviour
            #[derive(Builder)]
            struct RepeatBTreeMap {
                #[builder(repeat = (u32, u32))]
                repeat: BTreeMap<u32, u32>,
            }

            #[test]
            fn repeat_btreemap_0() {
                let rep = RepeatBTreeMap::builder().build();
                assert_eq!(rep.repeat, BTreeMap::new());
            }

            #[test]
            fn repeat_btreemap_2() {
                let rep = RepeatBTreeMap::builder()
                    .repeat((1, 2))
                    .repeat((3, 4))
                    .build();
                assert_eq!(rep.repeat, BTreeMap::from([(1, 2), (3, 4)]));
            }

            #[derive(Builder)]
            struct RepeatTupleInto {
                #[builder(repeat, tuple, into)]
                repeat: Vec<(Distinct<u32>, Distinct<u32>)>,
            }

            #[test]
            fn repeat_tuple_into_0() {
                let rep = RepeatTupleInto::builder().build();
                assert_eq!(rep.repeat, []);
            }

            #[test]
            fn repeat_tuple_into_2() {
                let rep = RepeatTupleInto::builder().repeat(1, 2).repeat(3, 4).build();
                assert_eq!(
                    rep.repeat,
                    [(Distinct(1), Distinct(2)), (Distinct(3), Distinct(4))]
                );
            }

            #[derive(Builder)]
            struct RepeatHashMapIntoTuple {
                #[builder(repeat = (Distinct<u32>, Distinct<u32>), into, tuple)]
                repeat: HashMap<Distinct<u32>, Distinct<u32>>,
            }

            #[test]
            fn repeat_hashmap_tuple_into_0() {
                let rep = RepeatHashMapIntoTuple::builder().build();
                assert_eq!(rep.repeat, HashMap::new());
            }

            #[test]
            fn repeat_hashmap_tuple_into_2() {
                let rep = RepeatHashMapIntoTuple::builder()
                    .repeat(1, 2)
                    .repeat(3, 4)
                    .build();
                assert_eq!(
                    rep.repeat,
                    HashMap::from([(Distinct(1), Distinct(2)), (Distinct(3), Distinct(4)),])
                );
            }

            fn sum(iter: impl Iterator<Item = u32>) -> Distinct<u32> {
                Distinct(iter.sum())
            }

            #[derive(Builder)]
            struct RepeatSum {
                #[builder(repeat = u32, collector = sum)]
                repeat: Distinct<u32>,
            }

            #[test]
            fn repeat_collector_0() {
                let rep = RepeatSum::builder().build();
                assert_eq!(rep.repeat, Distinct(0));
            }

            #[test]
            fn repeat_collector_2() {
                let rep = RepeatSum::builder().repeat(34).repeat(35).build();
                assert_eq!(rep.repeat, Distinct(69));
            }

            fn sum_distinct(iter: impl Iterator<Item = Distinct<u32>>) -> u32 {
                iter.map(|x| x.into_inner()).sum()
            }

            #[derive(Builder)]
            struct RepeatSumInto {
                #[builder(repeat = Distinct<u32>, into, collector = sum_distinct)]
                repeat: u32,
            }

            #[test]
            fn repeat_collector_into_0() {
                let rep = RepeatSumInto::builder().build();
                assert_eq!(rep.repeat, 0);
            }

            #[test]
            fn repeat_collector_into_2() {
                let rep = RepeatSumInto::builder().repeat(34).repeat(35).build();
                assert_eq!(rep.repeat, 69);
            }

            fn sum_tuple(iter: impl Iterator<Item = (Distinct<u32>, u32)>) -> u32 {
                iter.map(|(x, y)| x.into_inner() * y).sum()
            }

            #[derive(Builder)]
            struct RepeatSumTuple {
                #[builder(repeat = (Distinct<u32>, u32), tuple, collector = sum_tuple)]
                repeat: u32,
            }

            #[test]
            fn repeat_collector_tuple_0() {
                let rep = RepeatSumTuple::builder().build();
                assert_eq!(rep.repeat, 0);
            }

            #[test]
            fn repeat_collector_tuple_2() {
                let rep = RepeatSumTuple::builder()
                    .repeat(Distinct(17), 2)
                    .repeat(Distinct(7), 5)
                    .build();
                assert_eq!(rep.repeat, 69);
            }

            #[derive(Builder)]
            struct RepeatSumAdapter {
                #[builder(repeat = u32, adapter = |x: Distinct<u32>| x.into_inner(), collector = sum)]
                repeat: Distinct<u32>,
            }

            #[test]
            fn repeat_collector_adapter_0() {
                let rep = RepeatSumAdapter::builder().build();
                assert_eq!(rep.repeat, Distinct(0));
            }

            #[test]
            fn repeat_collector_adapter_2() {
                let rep = RepeatSumAdapter::builder()
                    .repeat(Distinct(34))
                    .repeat(Distinct(35))
                    .build();
                assert_eq!(rep.repeat, Distinct(69));
            }
        }
    };
}

tests!("borrowed" in mod borrowed);
tests!("owned" in mod owned);
tests!("type-state" in mod type_state);
