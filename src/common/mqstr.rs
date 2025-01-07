use std::{fmt::Display, ptr, str::FromStr};

use crate::sys;

use super::conversion;

pub type MqChar<const N: usize> = [sys::MQCHAR; N];

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
    pub const fn from_byte_slice(value: &[u8]) -> Result<Self, MqStrError> {
        Self::from_mqchar_slice(conversion::slice_byte_to_mqchar(value))
    }

    pub const fn from_mqchar_slice(value: &[sys::MQCHAR]) -> Result<Self, MqStrError> {
        let length = value.len();
        if N < length {
            return Err(MqStrError::Length { length, max: N });
        }
        let mut result = Self::empty();
        let mut i = 0;
        let l = [length, N][(length > N) as usize]; // Const trick to find the max value
        while i < l {
            result.data[i] = value[i];
            i += 1;
        }
        Ok(result)
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

    /// Use when defining `MqStr` from const or literal `&str`. Panics on invalid `MqStr`.
    #[must_use]
    pub const fn def_from_str(value: &str) -> Self {
        match Self::from_byte_slice(value.as_bytes()) {
            Ok(result) => result,
            Err(MqStrError::Length { .. }) => panic!("Invalid length"),
        }
    }

    /// The value of the `MqStr` without right padding
    #[must_use]
    pub fn value(&self) -> &[sys::MQCHAR] {
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

    pub fn assign(&mut self, value: impl AsRef<[sys::MQCHAR]>) -> bool {
        let mqchar_ref = value.as_ref();
        match self.data.split_at_mut_checked(mqchar_ref.len()) {
            Some((target, space)) => {
                target.copy_from_slice(mqchar_ref);
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
        Self::from_byte_slice(s.as_bytes())
    }
}

impl<const N: usize> Default for MqStr<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const N: usize> Display for MqStr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        String::from_utf8_lossy(unsafe { &*(ptr::from_ref(self.value()) as *const [u8]) }).fmt(f)
    }
}

impl<const N: usize> AsRef<MqChar<N>> for MqStr<N> {
    fn as_ref(&self) -> &MqChar<N> {
        &self.data
    }
}

impl<const N: usize> AsRef<[u8; N]> for MqStr<N> {
    fn as_ref(&self) -> &[u8; N] {
        self.as_bytes()
    }
}

impl<const N: usize> AsRef<MqStr<N>> for MqChar<N> {
    fn as_ref(&self) -> &MqStr<N> {
        unsafe { &*(ptr::addr_of!(self).cast()) }
    }
}

impl<const N: usize> AsRef<MqStr<N>> for [u8; N] {
    fn as_ref(&self) -> &MqStr<N> {
        unsafe { &*ptr::addr_of!(self).cast() }
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
        Self::from_byte_slice(value.as_bytes())
    }
}
