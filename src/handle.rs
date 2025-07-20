use std::fmt::Display;

use ::libmqm_constants::{
    lookup::{ConstLookup as _, HasConstLookup as _},
    mapping,
};
use libmqm_sys as mq;

/// Implements `HasConstLookup` using the provided `ConstSource` static instance
macro_rules! impl_constant_lookup {
    ($t:ty, $source:path) => {
        impl ::libmqm_constants::lookup::HasConstLookup for $t {
            fn const_lookup<'a>() -> &'a (impl ::libmqm_constants::lookup::ConstLookup + 'static) {
                &$source
            }
        }
    };
}

pub trait RawHandle {
    type HandleType: Copy;
}

pub mod raw {
    #[cfg(feature = "mqai")]
    use libmqm_sys::mqai;

    use super::{RawHandle, mq};

    #[derive(Debug, Clone, Copy)]
    pub struct Connection;
    impl RawHandle for Connection {
        type HandleType = mq::MQHCONN;
    }

    #[derive(Debug)]
    pub struct Message;
    impl RawHandle for Message {
        type HandleType = mq::MQHMSG;
    }

    #[derive(Debug)]
    pub struct Object;
    impl RawHandle for Object {
        type HandleType = mq::MQHOBJ;
    }

    #[derive(Debug)]
    #[cfg(feature = "mqai")]
    pub struct Bag;

    #[cfg(feature = "mqai")]
    impl RawHandle for Bag {
        type HandleType = mqai::MQHBAG;
    }
}

pub type ConnectionHandle = Handle<raw::Connection>;
pub type ObjectHandle = Handle<raw::Object>;
pub type MessageHandle = Handle<raw::Message>;
pub type SubscriptionHandle = ObjectHandle;

#[cfg(feature = "mqai")]
mod mqai {
    use libmqm_constants::lookup::{ConstLookup, HasConstLookup};
    use libmqm_sys::mqai;

    pub type BagHandle = super::Handle<super::raw::Bag>;

    impl From<mqai::MQHBAG> for BagHandle {
        fn from(value: mqai::MQHBAG) -> Self {
            Self(value)
        }
    }

    impl HasConstLookup for BagHandle {
        fn const_lookup<'a>() -> &'a (impl ConstLookup + 'static) {
            &super::mapping::MQHB_MAPSTR
        }
    }

    impl std::fmt::Display for BagHandle {
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
}

#[cfg(feature = "mqai")]
pub use mqai::*;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[repr(transparent)]
pub struct Handle<R: RawHandle>(pub(super) R::HandleType);

impl<R: RawHandle> Handle<R> {
    /// ## Safety
    /// Consumers of the function must use the handle directly on MQ API calls
    #[must_use]
    pub const fn raw_handle(&self) -> R::HandleType {
        self.0
    }

    /// ## Safety
    /// Primarily used by MQ to write/invalidate MQ handle
    #[must_use]
    pub const fn mut_raw_handle(&mut self) -> &mut R::HandleType {
        &mut self.0
    }
}

impl From<mq::MQHCONN> for ConnectionHandle {
    fn from(value: mq::MQHCONN) -> Self {
        Self(value)
    }
}

impl From<mq::MQHMSG> for MessageHandle {
    fn from(value: mq::MQHMSG) -> Self {
        Self(value)
    }
}

impl From<mq::MQHOBJ> for ObjectHandle {
    fn from(value: mq::MQHOBJ) -> Self {
        Self(value)
    }
}

impl ConnectionHandle {
    #[must_use]
    pub const fn is_disconnectable(&self) -> bool {
        self.0 != mq::MQHC_UNUSABLE_HCONN
    }
}

impl Default for ConnectionHandle {
    fn default() -> Self {
        Self(mq::MQHC_UNUSABLE_HCONN)
    }
}

impl_constant_lookup!(ConnectionHandle, mapping::MQHC_MAPSTR);

impl Display for ConnectionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match Self::const_lookup().by_value(self.0).next() {
            Some(name) => write!(f, "HCONN({name})"),
            None => write!(f, "HCONN({:#010X})", self.0),
        }
    }
}

impl MessageHandle {
    #[must_use]
    pub const fn is_deleteable(&self) -> bool {
        self.0 != mq::MQHM_UNUSABLE_HMSG
    }
}

impl ObjectHandle {
    #[must_use]
    pub const fn is_closeable(&self) -> bool {
        self.0 != mq::MQHO_UNUSABLE_HOBJ
    }
}

impl Default for ObjectHandle {
    fn default() -> Self {
        Self(mq::MQHO_UNUSABLE_HOBJ)
    }
}

impl_constant_lookup!(ObjectHandle, mapping::MQHO_MAPSTR);

impl Display for ObjectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match Self::const_lookup().by_value(self.0).next() {
            Some(name) => write!(f, "HOBJ({name})"),
            None => write!(f, "HOBJ({:#010X})", self.0),
        }
    }
}

impl_constant_lookup!(MessageHandle, mapping::MQHM_MAPSTR);

impl Display for MessageHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let handle_mqlong = self.0.try_into().ok();
        match handle_mqlong.and_then(|value| Self::const_lookup().by_value(value).next()) {
            Some(name) => write!(f, "HMSG({name})"),
            None => write!(f, "HMSG({:#018X})", self.0),
        }
    }
}

impl Default for MessageHandle {
    fn default() -> Self {
        Self(mq::MQHM_UNUSABLE_HMSG)
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;

    #[test]
    fn connection_handle_display() {
        assert_eq!(
            ConnectionHandle::from(mq::MQHC_DEF_HCONN).to_string(),
            "HCONN(MQHC_DEF_HCONN)"
        );
        assert_eq!(
            ConnectionHandle::from(mq::MQHC_UNUSABLE_HCONN).to_string(),
            "HCONN(MQHC_UNUSABLE_HCONN)"
        );
    }

    #[test]
    fn object_handle_display() {
        assert_eq!(ObjectHandle::from(mq::MQHO_NONE).to_string(), "HOBJ(MQHO_NONE)");
        assert_eq!(ObjectHandle::from(1).to_string(), "HOBJ(0x00000001)");
    }

    #[test]
    fn message_handle_display() {
        assert_eq!(MessageHandle::from(mq::MQHM_NONE).to_string(), "HMSG(MQHM_NONE)");
        assert_eq!(
            MessageHandle::from(mq::MQHM_UNUSABLE_HMSG).to_string(),
            "HMSG(MQHM_UNUSABLE_HMSG)"
        );
    }

    #[test]
    #[cfg(feature = "mqai")]
    fn bag_handle_display() {
        assert_eq!(BagHandle::default().to_string(), "HBAG(MQHB_UNUSABLE_HBAG)");
        assert_eq!(Into::<BagHandle>::into(1).to_string(), "HBAG(0x00000001)");
    }
}
