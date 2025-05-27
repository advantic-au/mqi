use std::mem;

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
