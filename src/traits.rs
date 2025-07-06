use std::{borrow::Cow, cmp, mem};

use libmqm_sys as mq;

pub trait Secret<'y, Y: ?Sized> {
    #[must_use]
    fn expose_secret(&self) -> &'y Y;
}

trait Sealed {}
#[expect(private_bounds, reason = "sealed trait pattern")]
pub trait MQMD: Sealed + std::fmt::Debug {}
impl Sealed for mq::MQMD {}
impl Sealed for mq::MQMD1 {}
impl Sealed for mq::MQMD2 {}

impl MQMD for mq::MQMD {}
impl MQMD for mq::MQMD1 {}
impl MQMD for mq::MQMD2 {}

/// A marker trait where it is safe to write arbitrary bytes
///
/// # Safety
/// Implementations of [`WriteRaw`] must ensure that writing arbitrary data into the value will not cause undefined behaviour
pub unsafe trait WriteRaw<T> {}

unsafe impl WriteRaw<Self> for u8 {}
unsafe impl WriteRaw<Self> for i8 {}
unsafe impl WriteRaw<Self> for i16 {}
unsafe impl WriteRaw<Self> for i32 {}
unsafe impl WriteRaw<Self> for i64 {}
unsafe impl<B: WriteRaw<T>, T> WriteRaw<T> for mem::MaybeUninit<B> {}
unsafe impl<B: WriteRaw<T>, T> WriteRaw<T> for [B] {}
unsafe impl<const N: usize, B: WriteRaw<T>, T> WriteRaw<T> for [B; N] {}

pub trait ReadRaw {}
impl ReadRaw for u8 {}
impl ReadRaw for i8 {}
impl ReadRaw for i32 {}
impl ReadRaw for i64 {}
impl ReadRaw for i16 {}
impl ReadRaw for f32 {}
impl ReadRaw for f64 {}
impl ReadRaw for str {}

impl<B: ReadRaw> ReadRaw for [B] {}
impl<const N: usize, B: ReadRaw> ReadRaw for [B; N] {}

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
