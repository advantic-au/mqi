use std::fmt::Display;

use crate::constants::ConstLookup as _;
use crate::constants::HasConstLookup as _;

use crate::sys;
use crate::{
    constants::mapping,
    impl_constant_lookup,
};

pub trait RawHandle {
    type HandleType: Copy;
}

pub mod raw {
    use super::RawHandle;
    use crate::sys;

    #[derive(Debug)]
    pub struct Connection;
    impl RawHandle for Connection {
        type HandleType = sys::MQHCONN;
    }

    #[derive(Debug)]
    pub struct Message;
    impl RawHandle for Message {
        type HandleType = sys::MQHMSG;
    }

    #[derive(Debug)]
    pub struct Object;
    impl RawHandle for Object {
        type HandleType = sys::MQHOBJ;
    }
}

pub type ConnectionHandle = Handle<raw::Connection>;
pub type ObjectHandle = Handle<raw::Object>;
pub type MessageHandle = Handle<raw::Message>;
pub type SubscriptionHandle = ObjectHandle;

#[derive(PartialEq, Eq, Debug, Clone)]
pub struct Handle<R: RawHandle>(pub(super) R::HandleType);

impl<R: RawHandle> Handle<R> {
    /// # Safety
    /// Consumers of the function must use the handle directly on MQ API calls
    #[must_use]
    pub const unsafe fn raw_handle(&self) -> R::HandleType {
        self.0
    }

    /// # Safety
    /// Primarily used by MQ to write/invalidate MQ handle
    #[must_use]
    pub unsafe fn mut_raw_handle(&mut self) -> &mut R::HandleType {
        &mut self.0
    }
}

impl From<sys::MQHCONN> for ConnectionHandle {
    fn from(value: sys::MQHCONN) -> Self {
        Self(value)
    }
}

impl From<sys::MQHMSG> for MessageHandle {
    fn from(value: sys::MQHMSG) -> Self {
        Self(value)
    }
}

impl From<sys::MQHOBJ> for ObjectHandle {
    fn from(value: sys::MQHOBJ) -> Self {
        Self(value)
    }
}

pub const UNNASSOCIATED_HCONN: ConnectionHandle = Handle(sys::MQHC_UNASSOCIATED_HCONN);

impl ConnectionHandle {
    #[must_use]
    pub const fn is_disconnectable(&self) -> bool {
        self.0 != sys::MQHC_UNUSABLE_HCONN
    }
}

impl Default for ConnectionHandle {
    fn default() -> Self {
        Self(sys::MQHC_UNUSABLE_HCONN)
    }
}

impl_constant_lookup!(ConnectionHandle, mapping::MQHC_CONST);

impl Display for ConnectionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Self::const_lookup().by_value(self.0).next() {
            Some(name) => write!(f, "HCONN({name})"),
            None => write!(f, "HCONN({:#010X})", self.0),
        }
    }
}

pub const MQHO_NONE: ObjectHandle = Handle(sys::MQHO_NONE);

impl ObjectHandle {
    #[must_use]
    pub const fn is_closeable(&self) -> bool {
        self.0 != sys::MQHO_UNUSABLE_HOBJ
    }
}

impl Default for ObjectHandle {
    fn default() -> Self {
        Self(sys::MQHO_UNUSABLE_HOBJ)
    }
}

impl_constant_lookup!(ObjectHandle, mapping::MQHO_CONST);

impl Display for ObjectHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match Self::const_lookup().by_value(self.0).next() {
            Some(name) => write!(f, "HOBJ({name})"),
            None => write!(f, "HOBJ({:#010X})", self.0),
        }
    }
}

impl_constant_lookup!(MessageHandle, mapping::MQHM_CONST);

impl Display for MessageHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let handle_mqlong = self.0.try_into().ok();
        match handle_mqlong.and_then(|value| Self::const_lookup().by_value(value).next()) {
            Some(name) => write!(f, "HMSG({name})"),
            None => write!(f, "HMSG({:#018X})", self.0),
        }
    }
}

impl Default for MessageHandle {
    fn default() -> Self {
        Self(sys::MQHM_UNUSABLE_HMSG)
    }
}

#[cfg(test)]
mod tests {
    use super::{ConnectionHandle, MessageHandle, ObjectHandle};
    use crate::sys;

    #[test]
    fn connection_handle_display() {
        assert_eq!(
            ConnectionHandle::from(sys::MQHC_DEF_HCONN).to_string(),
            "HCONN(MQHC_DEF_HCONN)"
        );
        assert_eq!(
            ConnectionHandle::from(sys::MQHC_UNUSABLE_HCONN).to_string(),
            "HCONN(MQHC_UNUSABLE_HCONN)"
        );
    }

    #[test]
    fn object_handle_display() {
        assert_eq!(ObjectHandle::from(sys::MQHO_NONE).to_string(), "HOBJ(MQHO_NONE)");
        assert_eq!(ObjectHandle::from(1).to_string(), "HOBJ(0x00000001)");
    }

    #[test]
    fn message_handle_display() {
        assert_eq!(MessageHandle::from(sys::MQHM_NONE).to_string(), "HMSG(MQHM_NONE)");
        assert_eq!(
            MessageHandle::from(sys::MQHM_UNUSABLE_HMSG).to_string(),
            "HMSG(MQHM_UNUSABLE_HMSG)"
        );
    }
}
