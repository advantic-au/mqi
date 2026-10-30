use std::{borrow::Cow, mem};

use crate::types;

const _: () =
    assert!(mem::size_of::<u8>() == mem::size_of::<types::MQCHAR>() && mem::align_of::<u8>() == mem::align_of::<types::MQCHAR>());

#[must_use]
pub fn vec_byte_to_mqchar(byte_vec: Vec<u8>) -> Vec<types::MQCHAR> {
    // Keep the allocation alive while changing the element type.
    let mut v = mem::ManuallyDrop::new(byte_vec);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // SAFETY: MQCHAR and u8 have equal size and alignment (asserted above), and
    // both integer types accept every bit pattern. The original Vec is not dropped.
    unsafe { Vec::from_raw_parts(p.cast(), len, cap) }
}

#[must_use]
pub fn vec_mqchar_to_byte(mqchar_vec: Vec<types::MQCHAR>) -> Vec<u8> {
    let mut v = mem::ManuallyDrop::new(mqchar_vec);
    let p = v.as_mut_ptr();
    let len = v.len();
    let cap = v.capacity();

    // SAFETY: u8 and MQCHAR have equal size and alignment (asserted above), and
    // both integer types accept every bit pattern. The original Vec is not dropped.
    unsafe { Vec::from_raw_parts(p.cast(), len, cap) }
}

#[inline]
#[must_use]
pub const fn slice_byte_to_mqchar(byte_slice: &[u8]) -> &[types::MQCHAR] {
    // SAFETY: u8 and MQCHAR have equal size and alignment (asserted above), and
    // both integer types accept every bit pattern.
    unsafe { mem::transmute(byte_slice) }
}

#[inline]
#[must_use]
pub const fn slice_mqchar_to_byte(mqchar_slice: &[types::MQCHAR]) -> &[u8] {
    // SAFETY: MQCHAR and u8 have equal size and alignment (asserted above), and
    // both integer types accept every bit pattern.
    unsafe { mem::transmute(mqchar_slice) }
}

pub fn bytes_to_cow_mqchar<'a, T: Into<Cow<'a, [u8]>>>(byte_cow: T) -> Cow<'a, [types::MQCHAR]> {
    match byte_cow.into() {
        Cow::Borrowed(s) => Cow::Borrowed(slice_byte_to_mqchar(s)),
        Cow::Owned(v) => Cow::Owned(vec_byte_to_mqchar(v)),
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn vec_conversions_preserve_bytes_and_capacity() {
        let mut bytes = Vec::with_capacity(16);
        bytes.extend([0, 0x7f, 0x80, 0xff]);
        let capacity = bytes.capacity();

        let mqchars = vec_byte_to_mqchar(bytes);
        assert_eq!(mqchars, slice_byte_to_mqchar(&[0, 0x7f, 0x80, 0xff]));
        assert_eq!(mqchars.capacity(), capacity);

        let bytes = vec_mqchar_to_byte(mqchars);
        assert_eq!(bytes, [0, 0x7f, 0x80, 0xff]);
        assert_eq!(bytes.capacity(), capacity);
    }

    #[test]
    fn slice_conversion_preserves_bytes() {
        let bytes = [0, 0x7f, 0x80, 0xff];
        let mqchars = slice_byte_to_mqchar(&bytes);

        assert_eq!(slice_mqchar_to_byte(mqchars), bytes);
    }

    #[test]
    fn cow_conversion_preserves_borrowed_and_owned_storage() {
        let bytes = [0, 0x80, 0xff];
        let borrowed = bytes_to_cow_mqchar(Cow::Borrowed(bytes.as_slice()));
        assert!(matches!(borrowed, Cow::Borrowed(_)));
        assert_eq!(slice_mqchar_to_byte(&borrowed), bytes);

        let owned = bytes_to_cow_mqchar(Cow::Owned(bytes.to_vec()));
        assert!(matches!(owned, Cow::Owned(_)));
        assert_eq!(slice_mqchar_to_byte(&owned), bytes);
    }

    proptest! {
        #[test]
        fn vec_conversions_round_trip_arbitrary_bytes(bytes in proptest::collection::vec(any::<u8>(), 0..256)) {
            prop_assert_eq!(vec_mqchar_to_byte(vec_byte_to_mqchar(bytes.clone())), bytes);
        }
    }
}
