use bauer::Builder;

macro_rules! define {
    // NOTE: using tt for $ty becuase ty seems to expand weirdly with the derive
    ($before: ident, $after: ident as ($($ty: tt)*) => $($tt: tt)*) => {
        #[derive(Builder)]
        struct $before {
            #[builder(skip, $($tt)*)]
            field: $($ty)*,
        }

        #[derive(Builder)]
        struct $after {
            #[builder($($tt)*, skip)]
            field: $($ty)*,
        }
    }
}

fn sum(iter: impl Iterator<Item = u32>) -> u32 {
    iter.sum()
}

define!(BeforeDefault,    AfterDefault    as (       u32) => default                       );
define!(BeforeRepeat,     AfterRepeat     as (  Vec<u32>) => repeat                        );
define!(BeforeRepeatN,    AfterRepeatN    as (  Vec<u32>) => repeat, repeat_n = 2          );
define!(BeforeCollector,  AfterCollector  as (       u32) => repeat = u32, collector = foo );
define!(BeforeInto,       AfterInto       as (       u32) => into                          );
define!(BeforeTuple,      AfterTuple      as ((u32, u32)) => tuple                         );
define!(BeforeAdapter,    AfterAdapter    as (       u32) => adapter = |n:u32| n + 2       );
define!(BeforeRename,     AfterRename     as (       u32) => rename = "renamed"            );
define!(BeforeSkipPrefix, AfterSkipPrefix as (       u32) => skip_prefix                   );
define!(BeforeSkipSuffix, AfterSkipSuffix as (       u32) => skip_suffix                   );
define!(BeforeAttributes, AfterAttributes as (       u32) => attributes()                  );
define!(BeforeDoc,        AfterDoc        as (       u32) => doc()                         );

fn main() {}
