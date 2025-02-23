use std::mem;

use crate::sys;

/// A marker trait where it is safe to write arbitrary bytes
///
/// # Safety
/// Implementations of [`WriteRaw`] must ensure that writing arbitrary data into the value will not cause undefined behaviour
pub unsafe trait WriteRaw<T> {}

unsafe impl WriteRaw<Self> for sys::MQBYTE {}
unsafe impl WriteRaw<Self> for sys::MQCHAR {}
unsafe impl WriteRaw<Self> for sys::MQLONG {}
unsafe impl WriteRaw<Self> for sys::MQINT64 {}
unsafe impl<B: WriteRaw<T>, T> WriteRaw<T> for mem::MaybeUninit<B> {}
unsafe impl<B: WriteRaw<T>, T> WriteRaw<T> for [B] {}
unsafe impl<const N: usize, B: WriteRaw<T>, T> WriteRaw<T> for [B; N] {}

pub trait ReadRaw {}
impl ReadRaw for sys::MQBYTE {}
impl ReadRaw for sys::MQCHAR {}
impl ReadRaw for sys::MQLONG {}
impl ReadRaw for sys::MQINT64 {}
impl ReadRaw for i16 {}
impl ReadRaw for f32 {}
impl ReadRaw for f64 {}
impl ReadRaw for str {}

impl<B: ReadRaw> ReadRaw for [B] {}
impl<const N: usize, B: ReadRaw> ReadRaw for [B; N] {}
