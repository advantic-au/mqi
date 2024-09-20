use libmqm_sys::link::LinkedMq;

use super::{connect_options::ConnectOption, ConnectAttr, ConnectValue, Connection, HandleShare};
use crate::{values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::admin::{Bag, Owned};

/// Create a [`Connection`] to a queue manager using the compile time linked MQ library
/// and inferred return value.
#[inline]
pub fn connect_as<'co, R, H>(options: impl ConnectOption<'co>) -> ResultComp<R>
where
    R: ConnectValue<Connection<LinkedMq, H>>,
    H: HandleShare,
{
    super::connect_lib_as(LinkedMq, options)
}

/// Create and return a [`Connection`] to a queue manager using the compile time linked MQ library.
#[inline]
pub fn connect<'co, H>(options: impl ConnectOption<'co>) -> ResultComp<Connection<LinkedMq, H>>
where
    H: HandleShare,
{
    super::connect_lib_as(LinkedMq, options)
}

/// Create and return a [`Connection`] to a queue manager using the compile time linked MQ library
/// and inferred [`ConnectAttr`].
#[inline]
pub fn connect_with<'co, A, H>(options: impl ConnectOption<'co>) -> ResultComp<(Connection<LinkedMq, H>, A)>
where
    A: ConnectAttr<Connection<LinkedMq, H>>,
    H: HandleShare,
{
    super::connect_lib_as(LinkedMq, options)
}

#[cfg(feature = "mqai")]
impl Bag<Owned, LinkedMq> {
    pub fn new(options: MQCBO) -> ResultComp<Self> {
        Self::new_lib(LinkedMq, options)
    }
}
