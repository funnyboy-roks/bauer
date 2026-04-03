use std::{marker::PhantomData, mem::MaybeUninit};

use crate::__private::sealed::Sealed;

// NOTE: These functions are not associated with the trait so that they may be const.

// SAFETY: If State::SET, then t _must_ have been initialised before calling this function.
pub const unsafe fn into_option<State: BuilderState, T>(t: MaybeUninit<T>) -> Option<T> {
    if State::SET {
        // SAFETY: Caller must ensure that `t` has been initialised.
        Some(unsafe { t.assume_init() })
    } else {
        None
    }
}

// SAFETY: If State::SET, then t _must_ have been initialised before calling this function.
pub unsafe fn unwrap_or_else<State: BuilderState, T, F>(t: MaybeUninit<T>, or_else: F) -> T
where
    F: FnOnce() -> T,
{
    if State::SET {
        // SAFETY: Caller must ensure that `t` has been initialised.
        unsafe { t.assume_init() }
    } else {
        or_else()
    }
}

pub trait BuilderState: Sealed {
    const SET: bool;
}

pub struct Count<T>(PhantomData<T>);
impl sealed::Sealed for Count<()> {}
impl<R, V> sealed::Sealed for Count<(R, V)> where Count<R>: Countable {}

mod sealed {
    /// Proper seal
    pub trait Sealed {}
}

pub trait Countable: sealed::Sealed {
    const COUNT: usize;
}

impl Countable for Count<()> {
    const COUNT: usize = 0;
}

impl<R, V> Countable for Count<(R, V)>
where
    Count<R>: Countable,
{
    const COUNT: usize = 1 + <Count<R>>::COUNT;
}

#[cfg(test)]
mod test {
    use std::mem::MaybeUninit;

    use crate::__private::{
        self,
        state::{BuilderState, into_option, unwrap_or_else},
    };

    struct FooState<const SET: bool>;
    impl<const SET: bool> __private::sealed::Sealed for FooState<SET> {}
    impl<const SET: bool> BuilderState for FooState<SET> {
        const SET: bool = SET;
    }

    #[test]
    fn into_option_set() {
        let option = unsafe { into_option::<FooState<true>, _>(MaybeUninit::new(69)) };
        assert_eq!(option, Some(69));
    }

    #[test]
    fn into_option_unset() {
        let option = unsafe { into_option::<FooState<false>, _>(MaybeUninit::<u32>::uninit()) };
        assert_eq!(option, None);
    }

    #[test]
    fn into_option_set_const() {
        let option = const { unsafe { into_option::<FooState<true>, _>(MaybeUninit::new(69)) } };
        assert_eq!(option, Some(69));
    }

    #[test]
    fn into_option_unset_const() {
        let option =
            const { unsafe { into_option::<FooState<false>, _>(MaybeUninit::<u32>::uninit()) } };
        assert_eq!(option, None);
    }

    #[test]
    fn unwrap_or_else_set() {
        let option =
            unsafe { unwrap_or_else::<FooState<true>, _, _>(MaybeUninit::<u32>::new(420), || 0) };
        assert_eq!(option, 420);
    }

    #[test]
    fn unwrap_or_else_unset() {
        let option = unsafe {
            unwrap_or_else::<FooState<false>, _, _>(MaybeUninit::<u32>::uninit(), || 1337)
        };
        assert_eq!(option, 1337);
    }
}
