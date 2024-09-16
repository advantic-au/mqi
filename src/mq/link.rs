use libmqm_sys::link::LinkedMq;

use super::{connect_options::ConnectOption, ConnectAttr, ConnectValue, Connection, HandleShare, QueueManager};
use crate::{core::values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::admin::{Bag, Owned};

impl<H: HandleShare> QueueManager<Connection<LinkedMq, H>> {
    /// Create a connection to a queue manager using the compile time linked MQ library
    /// and inferred return value.
    #[inline]
    pub fn connect_as<'co, R>(options: impl ConnectOption<'co>) -> ResultComp<R>
    where
        R: ConnectValue<Self>,
    {
        Self::connect_lib_as(LinkedMq, options)
    }

    /// Create and return a connection to a queue manager using the compile time linked MQ library.
    #[inline]
    pub fn connect<'co>(options: impl ConnectOption<'co>) -> ResultComp<Self> {
        Self::connect_lib_as(LinkedMq, options)
    }

    /// Create and return a connection to a queue manager using the compile time linked MQ library
    /// and inferred [`ConnectAttr`].
    #[inline]
    pub fn connect_with<'co, A>(options: impl ConnectOption<'co>) -> ResultComp<(Self, A)>
    where
        A: ConnectAttr<Self>,
    {
        Self::connect_lib_as(LinkedMq, options)
    }
}

#[cfg(feature = "mqai")]
impl Bag<Owned, LinkedMq> {
    pub fn new(options: MQCBO) -> ResultComp<Self> {
        Self::connect_lib(LinkedMq, options)
    }
}
