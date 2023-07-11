use std::fmt::Display;

use crate::core::Handle;
use crate::mapping;
use crate::constants;
use crate::impl_constant_lookup;
use crate::sys;

use crate::constants::HasConstLookup;
use crate::constants::ConstLookup;

pub mod raw {
    use crate::{core::RawHandle, sys};

    #[derive(Debug)]
    pub struct Bag;

    impl RawHandle for Bag {
        type HandleType = sys::MQHBAG;
    }
}

pub type BagHandle = Handle<raw::Bag>;

pub const MQHB_NONE: BagHandle = Handle(sys::MQHB_NONE);

impl From<sys::MQHBAG> for BagHandle {
    fn from(value: sys::MQHBAG) -> Self {
        Self(value)
    }
}

impl_constant_lookup!(BagHandle, mapping::MQHB_CONST);

impl Display for BagHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Self::const_lookup().by_value(self.0).next() {
            Some(name) => write!(f, "HBAG({name})"),
            None => write!(f, "HBAG({:#010X})", self.0),
        }
    }
}

impl Default for BagHandle {
    fn default() -> Self {
        Self(sys::MQHB_UNUSABLE_HBAG)
    }
}

impl BagHandle {
    #[must_use]
    pub const fn is_deletable(&self) -> bool {
        self.0 != sys::MQHB_NONE && self.0 != sys::MQHB_UNUSABLE_HBAG
    }
}

#[cfg(test)]
mod tests {
    use crate::core::mqai::BagHandle;

    #[test]
    fn bag_handle_display() {
        assert_eq!(BagHandle::default().to_string(), "HBAG(MQHB_UNUSABLE_HBAG)");
        assert_eq!(Into::<BagHandle>::into(1).to_string(), "HBAG(0x00000001)");
    }
}
