use std::{rc::Rc, sync::Arc};

#[cfg(feature = "dlopen2")]
use {dlopen2::wrapper::Container, libmqm_sys::dlopen2::MqWrapper};

#[cfg(feature = "link")]
use libmqm_sys::link;

#[derive(Debug, Clone, Copy)]
pub struct MqFunctions<L>(pub L);

/// Holds a smart pointer to a [`MqFunctions`]
pub trait Library {
    type MQ;

    fn lib(&self) -> &Self::MQ;
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
    #[must_use]
    fn lib(&self) -> &Self::MQ {
        self
    }
}

#[cfg(feature = "link")]
impl super::MqFunctions<link::LinkedMq> {
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
