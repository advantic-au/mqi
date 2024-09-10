use libmqm_sys::link::LinkedMQ;

use super::{connect_options::ConnectOption, ConnectAttr, ConnectValue, HandleShare, QueueManagerShare};
use crate::{core::values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::admin::{Bag, Owned};

impl<H: HandleShare> QueueManagerShare<'_, LinkedMQ, H> {
    /// Create a connection to a queue manager using the compile time linked MQ library
    /// and inferred return value.
    #[inline]
    pub fn connect_as<'co, R>(options: impl ConnectOption<'co>) -> ResultComp<R>
    where
        R: ConnectValue<Self>,
    {
        Self::connect_lib_as(LinkedMQ, options)
    }

    /// Create and return a connection to a queue manager using the compile time linked MQ library.
    #[inline]
    pub fn connect<'co>(options: impl ConnectOption<'co>) -> ResultComp<Self> {
        Self::connect_lib_as(LinkedMQ, options)
    }

    /// Create and return a connection to a queue manager using the compile time linked MQ library
    /// and inferred [`ConnectAttr`].
    #[inline]
    pub fn connect_with<'co, A>(options: impl ConnectOption<'co>) -> ResultComp<(Self, A)>
    where
        A: ConnectAttr<Self>,
    {
        Self::connect_lib_as(LinkedMQ, options)
    }
}

#[cfg(feature = "mqai")]
impl Bag<Owned, LinkedMQ> {
    pub fn new(options: MQCBO) -> ResultComp<Self> {
        Self::connect_lib(LinkedMQ, options)
    }
}
