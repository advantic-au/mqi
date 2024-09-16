#[cfg(feature = "link")]
use libmqm_sys::link;

#[derive(Debug, Clone)]
pub struct MqFunctions<L>(pub L);

/// Holds a smart pointer to a [`MqFunctions`]
pub trait Library: Clone {
    type MQ;

    fn lib(&self) -> &Self::MQ;
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
