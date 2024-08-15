#[cfg(feature = "link")]
use libmqm_sys::link;

#[derive(Debug, Clone)]
pub struct MQFunctions<L>(pub L);

/// Holds a smart pointer to a `MQFunctions`
pub trait Library: Clone {
    type MQ;

    fn lib(&self) -> &Self::MQ;
}

#[cfg(feature = "link")]
impl Library for link::LinkedMQ {
    type MQ = Self;
    
    #[inline]
    #[must_use]
    fn lib(&self) -> &Self::MQ {
        self
    }

}

#[cfg(feature = "link")]
impl super::MQFunctions<link::LinkedMQ> {
    /// A compile-time linked `MQFunctions`
    #[must_use]
    #[inline]
    pub const fn linked() -> Self {
        Self(link::LinkedMQ)
    }
}
