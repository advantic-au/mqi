use std::{fmt::Display, ptr, str::FromStr};

use super::conversion;
use crate::{CCSID, EncodedString, types};

pub type MqChar<const N: usize> = [types::MQCHAR; N];

/// Fixed width string with trailing white space/nulls commonly
/// used with IBM MQ API's
#[derive(Debug, Eq, Clone, Copy, derive_more::AsMut)]
#[repr(transparent)]
pub struct MqStr<const N: usize> {
    #[as_mut]
    data: MqChar<N>,
}

/// Define a `MqStr` from constant `&str`.
#[macro_export]
macro_rules! mqstr {
    ($val:expr) => {
        const { $crate::MqStr::def_from_str($val) }
    };
}

impl<const N: usize> std::hash::Hash for MqStr<N> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value().hash(state);
    }
}

impl<const N: usize, const Y: usize> PartialEq<MqStr<Y>> for MqStr<N> {
    fn eq(&self, other: &MqStr<Y>) -> bool {
        self.value() == other.value()
    }
}

impl<const N: usize> PartialEq<&str> for MqStr<N> {
    fn eq(&self, other: &&str) -> bool {
        self.value() == other.data()
    }
}

impl<const N: usize> From<MqStr<N>> for MqChar<N> {
    fn from(mqstr: MqStr<N>) -> Self {
        *mqstr.as_ref()
    }
}

impl<const N: usize> From<MqChar<N>> for MqStr<N> {
    fn from(value: MqChar<N>) -> Self {
        *value.as_ref()
    }
}

impl<const N: usize, const Y: usize> PartialOrd<MqStr<Y>> for MqStr<N> {
    fn partial_cmp(&self, other: &MqStr<Y>) -> Option<std::cmp::Ordering> {
        self.value().partial_cmp(other.value())
    }
}

impl<const N: usize> MqStr<N> {
    pub const fn from_mqchar_slice(value: &[types::MQCHAR]) -> Result<&Self, MqStrError> {
        match value.split_first_chunk::<N>() {
            Some((val, _)) => Ok(unsafe { &*val.as_ptr().cast() }),
            None => Err(MqStrError::Length {
                length: value.len(),
                max: N,
            }),
        }
    }

    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; N] {
        unsafe { &*ptr::from_ref(self).cast() }
    }

    #[must_use]
    pub const fn as_mqchar(&self) -> &MqChar<N> {
        &self.data
    }

    /// Create an empty `MqStr` filled with spaces
    #[must_use]
    pub const fn empty() -> Self {
        Self { data: [0x20; N] } // Initialise with spaces
    }

    pub const fn from_str(value: &str) -> Result<Self, MqStrError> {
        let length = value.len();
        if length <= N {
            let mut result = Self::empty();
            let l = [length, N][(length > N) as usize];
            let (target, _) = unsafe { result.data.split_at_mut_unchecked(l) };
            let (source, _) = unsafe { value.as_bytes().split_at_unchecked(l) };
            target.copy_from_slice(conversion::slice_byte_to_mqchar(source));
            Ok(result)
        } else {
            Err(MqStrError::Length { length, max: N })
        }
    }

    /// Use when defining `MqStr` from const or literal `&str`. Panics on invalid `MqStr`.
    #[must_use]
    pub const fn def_from_str(value: &str) -> Self {
        match Self::from_str(value) {
            Ok(value) => value,
            Err(_) => panic!("value length exceeded MqStr length"),
        }
    }

    /// The value of the `MqStr` without right padding
    #[must_use]
    pub fn value(&self) -> &[types::MQCHAR] {
        let mut last = N;
        for _ in self.data.iter().rev().take_while(|c| **c == 0x20 || **c == 0) {
            last -= 1;
        }
        &self.data[..last]
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|c| *c == 0x20 || *c == 0)
    }

    #[must_use]
    pub fn has_value(&self) -> bool {
        self.data.iter().any(|c| *c != 0x20 && *c != 0)
    }

    pub fn assign(&mut self, value: &[types::MQCHAR]) -> bool {
        match self.data.split_at_mut_checked(value.len()) {
            Some((target, space)) => {
                target.copy_from_slice(value);
                space.fill(0x20);
                true
            }
            None => false,
        }
    }
}

impl<const N: usize> FromStr for MqStr<N> {
    type Err = MqStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_str(s)
    }
}

impl<const N: usize> Default for MqStr<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const N: usize> Display for MqStr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        String::from_utf8_lossy(unsafe { &*(ptr::from_ref(self.value()) as *const _) }).fmt(f)
    }
}

impl<const N: usize> AsRef<MqChar<N>> for MqStr<N> {
    fn as_ref(&self) -> &MqChar<N> {
        &self.data
    }
}

impl<const N: usize> AsRef<MqStr<N>> for MqChar<N> {
    fn as_ref(&self) -> &MqStr<N> {
        unsafe { &*self.as_ptr().cast() }
    }
}

impl<const N: usize> AsMut<MqStr<N>> for MqChar<N> {
    fn as_mut(&mut self) -> &mut MqStr<N> {
        unsafe { &mut *self.as_mut_ptr().cast() }
    }
}

#[derive(derive_more::Error, derive_more::Display, Debug)]
pub enum MqStrError {
    #[display("String of length {length} exceeds maximum length {max}")]
    Length { length: usize, max: usize },
}

impl<const N: usize> TryFrom<&str> for MqStr<N> {
    type Error = MqStrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_str(value)
    }
}

impl<const N: usize> EncodedString for MqStr<N> {
    fn ccsid(&self) -> CCSID {
        CCSID(1208)
    }

    fn data(&self) -> &[types::MQCHAR] {
        &self.data
    }
}
