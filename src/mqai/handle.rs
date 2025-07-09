use std::fmt::Display;

use libmqm_constants::{
    lookup::{ConstLookup, HasConstLookup},
    mapping,
};
use libmqm_sys::mqai;

use crate::Handle;

mod raw {
    use super::mqai;
    use crate::RawHandle;

    #[derive(Debug)]
    pub struct Bag;

    impl RawHandle for Bag {
        type HandleType = mqai::MQHBAG;
    }
}

pub type BagHandle = Handle<raw::Bag>;

impl From<mqai::MQHBAG> for BagHandle {
    fn from(value: mqai::MQHBAG) -> Self {
        Self(value)
    }
}

impl HasConstLookup for BagHandle {
    fn const_lookup<'a>() -> &'a (impl ConstLookup + 'static) {
        &mapping::MQHB_MAPSTR
    }
}

impl Display for BagHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match Self::const_lookup().by_value(self.0).next() {
            Some(name) => write!(f, "HBAG({name})"),
            None => write!(f, "HBAG({:#010X})", self.0),
        }
    }
}

impl Default for BagHandle {
    fn default() -> Self {
        Self(mqai::MQHB_UNUSABLE_HBAG)
    }
}

impl BagHandle {
    #[must_use]
    pub const fn is_deletable(&self) -> bool {
        self.0 != mqai::MQHB_NONE && self.0 != mqai::MQHB_UNUSABLE_HBAG
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn bag_handle_display() {
        assert_eq!(BagHandle::default().to_string(), "HBAG(MQHB_UNUSABLE_HBAG)");
        assert_eq!(Into::<BagHandle>::into(1).to_string(), "HBAG(0x00000001)");
    }
}
