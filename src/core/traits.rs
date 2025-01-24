use std::mem;

use crate::sys;

/// A marker trait where it is safe to write arbitrary bytes
///
/// # Safety
/// Implementations of [`WriteByte`] must ensure that writing arbitrary data into the value will not cause undefined behaviour
pub unsafe trait WriteByte<T> {}

unsafe impl WriteByte<Self> for sys::MQBYTE {}
unsafe impl WriteByte<Self> for sys::MQCHAR {}
unsafe impl WriteByte<Self> for sys::MQLONG {}
unsafe impl WriteByte<Self> for sys::MQINT64 {}
unsafe impl<B: WriteByte<T>, T> WriteByte<T> for mem::MaybeUninit<B> {}
unsafe impl<B: WriteByte<T>, T> WriteByte<T> for [B] {}
unsafe impl<const N: usize, B: WriteByte<T>, T> WriteByte<T> for [B; N] {}
