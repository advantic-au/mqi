use std::{borrow::Cow, mem};

use crate::types;

#[must_use]
pub fn vec_byte_to_mqchar(byte_vec: Vec<u8>) -> Vec<types::MQCHAR> {
    // first, make sure v's destructor doesn't free the data
    // it thinks it owns when it goes out of scope
    let mut v = mem::ManuallyDrop::new(byte_vec);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p.cast(), len, cap) }
}

#[must_use]
pub fn vec_mqchar_to_byte(mqchar_vec: Vec<types::MQCHAR>) -> Vec<u8> {
    // first, make sure v's destructor doesn't free the data
    // it thinks it owns when it goes out of scope
    let mut v = mem::ManuallyDrop::new(mqchar_vec);

    // then, pick apart the existing Vec
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // finally, adopt the data into a new Vec
    unsafe { Vec::from_raw_parts(p.cast(), len, cap) }
}

#[inline]
#[must_use]
pub const fn slice_byte_to_mqchar(byte_slice: &[u8]) -> &[types::MQCHAR] {
    unsafe { mem::transmute(byte_slice) }
}

#[inline]
#[must_use]
pub const fn slice_mqchar_to_byte(mqchar_slice: &[types::MQCHAR]) -> &[u8] {
    unsafe { mem::transmute(mqchar_slice) }
}

pub fn bytes_to_cow_mqchar<'a, T: Into<Cow<'a, [u8]>>>(byte_cow: T) -> Cow<'a, [types::MQCHAR]> {
    match byte_cow.into() {
        Cow::Borrowed(s) => Cow::Borrowed(slice_byte_to_mqchar(s)),
        Cow::Owned(v) => Cow::Owned(vec_byte_to_mqchar(v)),
    }
}
