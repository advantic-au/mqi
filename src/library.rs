use std::{rc::Rc, sync::Arc};

#[cfg(feature = "link")]
use libmqm_sys::link;
#[cfg(feature = "dlopen2")]
use {dlopen2::wrapper::Container, libmqm_sys::dlopen2::MqWrapper};

/// A set of wrapper functions on the native IBM MQ API's
///
/// The functions provided wrap the native functions in [`Mqi`](libmqm_sys::Mqi) and [`Mqai`](libmqm_sys::Mqai) with additional functionality:
/// * [`MQRC`](crate::types::MQRC) and [`MQCC`](crate::types::MQCC) and return values are returned in a [`ResultComp`](crate::result::ResultComp)
/// * [`Tracing`](::tracing) capabilities
#[derive(Debug, Clone, Copy)]
pub struct MqFunctions<L>(pub L);

/// Holds a reference to an implementation of an MQ library
pub trait Library {
    type MQ;

    fn lib(&self) -> &Self::MQ;
}

#[cfg(feature = "mqai")]
pub trait MqaiLibrary {
    type MQAI: libmqm_sys::Mqai;

    fn lib(&self) -> &Self::MQAI;
}

impl<L: Library> Library for &L {
    type MQ = L::MQ;

    fn lib(&self) -> &Self::MQ {
        (*self).lib()
    }
}

impl<L: Library> Library for Rc<L> {
    type MQ = L::MQ;

    fn lib(&self) -> &Self::MQ {
        self.as_ref().lib()
    }
}

impl<L: Library> Library for Arc<L> {
    type MQ = L::MQ;

    fn lib(&self) -> &Self::MQ {
        self.as_ref().lib()
    }
}

#[cfg(feature = "link")]
impl Library for link::LinkedMq {
    type MQ = Self;

    #[inline]
    fn lib(&self) -> &Self::MQ {
        self
    }
}

#[cfg(feature = "mqai")]
impl<L: MqaiLibrary> MqaiLibrary for &L {
    type MQAI = L::MQAI;

    fn lib(&self) -> &Self::MQAI {
        (*self).lib()
    }
}

#[cfg(feature = "mqai")]
impl<L: MqaiLibrary> MqaiLibrary for Rc<L> {
    type MQAI = L::MQAI;

    fn lib(&self) -> &Self::MQAI {
        self.as_ref().lib()
    }
}

#[cfg(feature = "mqai")]
impl<L: MqaiLibrary> MqaiLibrary for Arc<L> {
    type MQAI = L::MQAI;

    fn lib(&self) -> &Self::MQAI {
        self.as_ref().lib()
    }
}

#[cfg(all(feature = "link", feature = "mqai"))]
impl MqaiLibrary for link::LinkedMq {
    type MQAI = Self;

    fn lib(&self) -> &Self::MQAI {
        self
    }
}

#[cfg(feature = "link")]
impl MqFunctions<link::LinkedMq> {
    /// A compile-time linked [`MqFunctions`]
    #[must_use]
    #[inline]
    pub const fn linked() -> Self {
        Self(link::LinkedMq)
    }
}

#[cfg(feature = "dlopen2")]
impl Library for Container<MqWrapper> {
    type MQ = Self;

    fn lib(&self) -> &Self::MQ {
        self
    }
}

#[cfg(all(feature = "dlopen2", feature = "mqai"))]
impl MqaiLibrary for Container<MqWrapper> {
    type MQAI = Self;

    fn lib(&self) -> &Self::MQAI {
        self
    }
}
