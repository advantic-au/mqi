use std::{borrow::Cow, cmp};

pub trait Buffer<'a, T>: Sized + AsMut<[T]> + AsRef<[T]> {
    #[must_use]
    fn truncate(self, size: usize) -> Self;
    fn split_at(self, at: usize) -> (Self, Self);
    fn into_cow(self) -> Cow<'a, [T]>
    where
        [T]: ToOwned,
        T: Clone;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a, T> Buffer<'a, T> for &'a mut [T] {
    fn truncate(self, size: usize) -> Self {
        let len = self.len();
        &mut self[..cmp::min(size, len)]
    }

    fn into_cow(self) -> Cow<'a, [T]>
    where
        T: Clone,
    {
        Cow::Borrowed(&*self)
    }

    fn len(&self) -> usize {
        (**self).len()
    }

    fn split_at(self, at: usize) -> (Self, Self) {
        self.split_at_mut(at)
    }
}

impl<'a, T: Clone> Buffer<'a, T> for Vec<T> {
    fn truncate(self, size: usize) -> Self {
        let mut vec = self;
        Self::truncate(&mut vec, size);
        vec.shrink_to_fit();
        vec
    }

    fn into_cow(self) -> Cow<'a, [T]> {
        self.into()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn split_at(self, at: usize) -> (Self, Self) {
        if at == 0 {
            (Self::new(), self) // No allocation when position is 0
        } else {
            let mut self_mut = self;
            let tail = self_mut.split_off(at);
            (self_mut, tail)
        }
    }
}
