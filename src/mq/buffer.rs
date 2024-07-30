use std::{
    borrow::{self, Cow},
    cmp,
};

pub enum InqBuffer<'a, T> {
    Slice(&'a mut [T]),
    Owned(Vec<T>),
}

impl<'a, T> InqBuffer<'a, T> {
    #[must_use]
    pub fn truncate(self, len: usize) -> Self {
        match self {
            Self::Slice(s) => {
                let buf_len = s.len();
                Self::Slice(&mut s[..cmp::min(len, buf_len)])
            }
            Self::Owned(mut v) => {
                v.truncate(len);
                Self::Owned(v)
            }
        }
    }
}

impl<'a, T> AsRef<[T]> for InqBuffer<'a, T> {
    fn as_ref(&self) -> &[T] {
        match self {
            InqBuffer::Slice(s) => s,
            InqBuffer::Owned(o) => o,
        }
    }
}

impl<'a, T> AsMut<[T]> for InqBuffer<'a, T> {
    fn as_mut(&mut self) -> &mut [T] {
        match self {
            InqBuffer::Slice(s) => s,
            InqBuffer::Owned(o) => o,
        }
    }
}

impl<'a, T: Clone> From<InqBuffer<'a, T>> for borrow::Cow<'a, [T]>
where
    [T]: ToOwned,
{
    fn from(value: InqBuffer<'a, T>) -> Self {
        match value {
            InqBuffer::Slice(s) => borrow::Cow::from(&*s),
            InqBuffer::Owned(o) => o.into(),
        }
    }
}

pub trait Buffer<'a>: Sized + AsMut<[u8]> + AsRef<[u8]> {
    #[must_use]
    fn truncate(self, size: usize) -> Self;
    fn split_at(self, at: usize) -> (Self, Self);
    fn into_cow(self) -> Cow<'a, [u8]>;
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<'a> Buffer<'a> for &'a mut [u8] {
    fn truncate(self, size: usize) -> Self {
        let len = self.len();
        &mut self[..cmp::min(size, len)]
    }

    fn into_cow(self) -> Cow<'a, [u8]> {
        Cow::from(&*self)
    }

    fn len(&self) -> usize {
        (**self).len()
    }
    
    fn split_at(self, at: usize) -> (Self, Self) {
        self.split_at_mut(at)
    }    
}

impl<'a> Buffer<'a> for Vec<u8> {
    fn truncate(self, size: usize) -> Self {
        let mut vec = self;
        Self::truncate(&mut vec, size);
        vec.shrink_to_fit();
        vec
    }

    fn into_cow(self) -> Cow<'a, [u8]> {
        self.into()
    }

    fn len(&self) -> usize {
        self.len()
    }
    
    fn split_at(self, at: usize) -> (Self, Self) {
        if at == 0 {
            (Vec::new(), self) // No allocation when position is 0
        } else {
            let mut self_mut = self;
            let tail = self_mut.split_off(at);
            (self_mut, tail)
        }
    }    
}

impl<'a> Buffer<'a> for InqBuffer<'a, u8> {
    fn truncate(self, size: usize) -> Self {
        Self::truncate(self, size)
    }

    fn into_cow(self) -> Cow<'a, [u8]> {
        self.into()
    }

    fn len(&self) -> usize {
        self.as_ref().len()
    }
    
    fn split_at(self, at: usize) -> (Self, Self) {
        match self {
            Self::Slice(s) => {
                let (head, tail) = s.split_at(at);
                (Self::Slice(head), Self::Slice(tail))
            },
            Self::Owned(v) => {
                let (head, tail) = v.split_at(at);
                (Self::Owned(head), Self::Owned(tail))
            },
        }
    }
}
