use std::{
    fmt::Display,
    mem::transmute,
    ptr::{addr_of, addr_of_mut},
    str::FromStr,
};

use crate::sys;

/// Fixed width string with trailing white space/nulls commonly
/// used with IBM MQ API's
#[derive(Debug, Eq, Clone, Copy)]
#[repr(transparent)]
pub struct MqStr<const N: usize> {
    data: [u8; N],
}

/// Define a `MqStr` from constant `&str`.
#[macro_export]
macro_rules! mqstr {
    ($val:expr) => {
        const { $crate::MqStr::from_str($val) }
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

impl<const N: usize> PartialEq<str> for MqStr<N> {
    fn eq(&self, other: &str) -> bool {
        self.value().eq(other.as_bytes())
    }
}

impl<const N: usize> From<MqStr<N>> for [sys::MQCHAR; N] {
    fn from(MqStr { data }: MqStr<N>) -> Self {
        unsafe { *addr_of!(data).cast() }
    }
}

impl<const N: usize> Ord for MqStr<N> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let str_self = unsafe { std::str::from_utf8_unchecked(self.value()) };
        let str_other = unsafe { std::str::from_utf8_unchecked(other.value()) };
        str_self.cmp(str_other)
    }
}

impl<const N: usize, const Y: usize> PartialOrd<MqStr<Y>> for MqStr<N> {
    fn partial_cmp(&self, other: &MqStr<Y>) -> Option<std::cmp::Ordering> {
        let str_other = unsafe { std::str::from_utf8_unchecked(other.value()) };
        self.partial_cmp(str_other)
    }
}

impl<const N: usize> PartialOrd<str> for MqStr<N> {
    fn partial_cmp(&self, other: &str) -> Option<std::cmp::Ordering> {
        let str_self = unsafe { std::str::from_utf8_unchecked(self.value()) };
        str_self.partial_cmp(other)
    }
}

impl<const N: usize> From<[sys::MQCHAR; N]> for MqStr<N> {
    fn from(value: [sys::MQCHAR; N]) -> Self {
        unsafe { *addr_of!(value).cast() }
    }
}

impl<const N: usize> MqStr<N> {
    pub const fn from_bytes(value: &[u8]) -> Result<Self, MQStrError> {
        let length = value.len();
        if N < length {
            return Err(MQStrError::Length { length, max: N });
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
        &self.data
    }

    #[must_use]
    pub const fn as_mqchar(&self) -> &[sys::MQCHAR; N] {
        unsafe { transmute(&self.data) }
    }

    /// Create an empty `MqStr` filled with spaces
    #[must_use]
    pub const fn empty() -> Self {
        Self { data: [b' '; N] } // Initialise with spaces
    }

    pub fn copy_into_mqchar(&self, target: &mut [sys::MQCHAR; N]) {
        self.as_ref().clone_into(target);
    }

    /// Use when defining `MqStr` from const or literal `&str`. Panics on invalid `MqStr`.
    #[must_use]
    pub const fn from_str(value: &str) -> Self {
        match Self::from_bytes(value.as_bytes()) {
            Ok(result) => result,
            Err(MQStrError::Length { .. }) => panic!("Invalid length"),
        }
    }

    /// The value of the `MqStr` without right padding
    #[must_use]
    pub fn value(&self) -> &[u8] {
        let mut last = N;
        for _ in self.data.iter().rev().take_while(|&c| *c == b' ' || *c == 0) {
            last -= 1;
        }
        &self.data[..last]
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|&c| c == b' ' || c == 0)
    }

    #[must_use]
    pub fn has_value(&self) -> bool {
        self.data.iter().any(|&c| c != b' ' && c != 0)
    }
}

impl<const N: usize> FromStr for MqStr<N> {
    type Err = MQStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_bytes(s.as_bytes())
    }
}

impl<const N: usize> Default for MqStr<N> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<const N: usize> Display for MqStr<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unsafe { std::str::from_utf8_unchecked(self.value()) }.fmt(f) // TODO: is unsafe ok here?
    }
}

impl<const N: usize> AsRef<[sys::MQCHAR; N]> for MqStr<N> {
    fn as_ref(&self) -> &[sys::MQCHAR; N] {
        unsafe { &*(addr_of!(self.data).cast()) }
    }
}

impl<const N: usize> AsRef<MqStr<N>> for [sys::MQCHAR; N] {
    fn as_ref(&self) -> &MqStr<N> {
        unsafe { &*(addr_of!(self).cast()) }
    }
}

impl<const N: usize> AsMut<[sys::MQCHAR; N]> for MqStr<N> {
    fn as_mut(&mut self) -> &mut [sys::MQCHAR; N] {
        unsafe { &mut *(addr_of_mut!(self.data).cast()) }
    }
}

#[derive(derive_more::Error, derive_more::Display, Debug)]
pub enum MQStrError {
    #[display("String of length {length} exceeds maximum length {max}")]
    Length { length: usize, max: usize },
}

impl<const N: usize> TryFrom<&str> for MqStr<N> {
    type Error = MQStrError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::from_bytes(value.as_bytes())
    }
}
