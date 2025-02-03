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

pub trait ReadByte {}
impl ReadByte for sys::MQBYTE {}
impl ReadByte for sys::MQCHAR {}
impl ReadByte for sys::MQLONG {}
impl ReadByte for sys::MQINT64 {}
impl ReadByte for i16 {}
impl ReadByte for f32 {}
impl ReadByte for f64 {}
impl ReadByte for str {}

impl<B: ReadByte> ReadByte for [B] {}
impl<const N: usize, B: ReadByte> ReadByte for [B; N] {}
