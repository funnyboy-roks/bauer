use std::mem::MaybeUninit;

pub mod state;

pub mod sealed {
    /// This is not _technically_ sealed, but a used would have to manually implement this trait,
    /// so it's good enough
    pub trait Sealed {}
}

#[derive(Debug)]
pub struct PushableArray<const N: usize, T> {
    len: usize,
    array: [MaybeUninit<T>; N],
}

impl<const N: usize, T> Default for PushableArray<N, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, T> PushableArray<N, T> {
    pub const fn new() -> Self {
        Self {
            array: [const { MaybeUninit::uninit() }; N],
            len: 0,
        }
    }

    /// Push a value.  Returns `Err(T)` if the value would not fit in the array
    // NOTE: This returns the failed T so that it can be const without needing `const Drop`
    pub const fn push(&mut self, t: T) -> Result<(), T> {
        let ret = if self.len < N {
            self.array[self.len].write(t);
            Ok(())
        } else {
            Err(t)
        };
        self.len += 1;
        ret
    }

    pub const fn len(&self) -> usize {
        self.len
    }

    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub const fn is_valid(&self) -> bool {
        self.len == N
    }

    pub const fn has_too_many(&self) -> bool {
        self.len > N
    }

    pub const fn into_array(self) -> Option<[T; N]> {
        if self.is_valid() {
            const {
                assert!(
                    std::mem::size_of::<[T; N]>() == std::mem::size_of::<[MaybeUninit<T>; N]>()
                );
            }
            // SAFETY: If self.len is N, then N items have been pushed and every item in the array
            // has been initalised.
            // transmute_copy is safe here since we know that both the MaybeUninit<T> array and the
            // T array are the same size.
            let array = unsafe { std::mem::transmute_copy(&self.array) };
            Some(array)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::__private::PushableArray;

    #[test]
    fn pushable_array() {
        let mut a = PushableArray::<2, u32>::new();
        let _ = a.push(69);
        let _ = a.push(420);
        assert!(a.is_valid());
        assert!(!a.has_too_many());
        let a = a.into_array().unwrap();
        assert_eq!(a, [69, 420]);
    }

    #[test]
    fn pushable_array_too_many() {
        let mut a = PushableArray::<2, u32>::new();
        let _ = a.push(69);
        let _ = a.push(420);
        let _ = a.push(1337);
        assert!(!a.is_valid());
        assert!(a.has_too_many());
        assert_eq!(a.into_array(), None);
    }

    #[test]
    fn pushable_array_too_few() {
        let mut a = PushableArray::<2, u32>::new();
        let _ = a.push(69);
        assert!(!a.is_valid());
        assert!(!a.has_too_many());
        assert_eq!(a.into_array(), None);
    }
}
