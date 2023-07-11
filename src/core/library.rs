use libmqm_sys::{function, MQI};

#[cfg(feature = "link")]
use libmqm_sys::link;

#[derive(Debug, Clone)]
pub struct MQFunctions<L>(pub L);

/// Holds a smart pointer to a `MQFunctions`
pub trait Library: std::ops::Deref<Target = Self::MQ> + Clone {
    type MQ: function::MQI;
}

impl<T: MQI> Library for &T {
    type MQ = T;
}

impl<T: MQI> Library for std::sync::Arc<T> {
    type MQ = T;
}

impl<T: MQI> Library for std::rc::Rc<T> {
    type MQ = T;
}

#[cfg(feature = "link")]
impl super::MQFunctions<&link::LinkedMQ> {
    /// A compile-time linked `MQFunctions`
    #[must_use]
    pub const fn linked() -> Self {
        Self(&link::LinkedMQ)
    }
}
