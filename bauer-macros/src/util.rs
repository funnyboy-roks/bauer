use quote::ToTokens;

pub(crate) struct Replace<I: Iterator + Sized> {
    iter: std::iter::Enumerate<I>,
    value: Option<<I as Iterator>::Item>,
    i: usize,
}

impl<I: Iterator> Iterator for Replace<I> {
    type Item = <I as Iterator>::Item;

    fn next(&mut self) -> Option<Self::Item> {
        let (i, next) = self.iter.next()?;
        if i == self.i {
            Some(self.value.take().expect("i == self.i only occurs once"))
        } else {
            Some(next)
        }
    }
}

pub(crate) trait ReplaceTrait: Iterator + Sized {
    fn replace(self, index: usize, value: Self::Item) -> Replace<Self>;
}

impl<I: Iterator> ReplaceTrait for I {
    fn replace(self, index: usize, value: Self::Item) -> Replace<Self> {
        Replace {
            iter: self.enumerate(),
            value: Some(value),
            i: index,
        }
    }
}

pub struct OptionalToken<T>(pub Option<T>);

impl<T> ToTokens for OptionalToken<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if let Some(t) = &self.0 {
            t.to_tokens(tokens)
        }
    }
}
