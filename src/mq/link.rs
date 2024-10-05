use libmqm_sys::link::LinkedMq;

use super::{connect_options::ConnectOption, ConnectAttr, ConnectValue, Connection, Threading};
use crate::{values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::admin::{Bag, Owned};

/// Create a [`Connection`] to a queue manager using the compile time linked MQ library
/// and type inferred [`ConnectValue`].
#[inline]
pub fn connect_as<'co, R, H>(options: impl ConnectOption<'co>) -> ResultComp<R>
where
    R: ConnectValue<Connection<LinkedMq, H>>,
    H: Threading,
{
    super::connect_lib_as(LinkedMq, options)
}

/// Create and return a [`Connection`] to a queue manager using the compile time linked MQ library.
///
/// # Examples
///
/// ```no_run
/// use mqi::prelude::*;
/// use mqi::{ThreadNone, connect_options::Credentials};
///
/// // Connect to the default queue manager with the provided credentials
/// let connection = mqi::connect::<ThreadNone>(Credentials::user("app", "app"))?;
///
/// // connection is wrapped in a Completion. Discard the completion with a `discard_warning`
/// let connection = connection.discard_warning();
///
/// # Ok::<(), mqi::Error>(())
/// ```
///
#[inline]
pub fn connect<'co, H>(options: impl ConnectOption<'co>) -> ResultComp<Connection<LinkedMq, H>>
where
    H: Threading,
{
    super::connect_lib_as(LinkedMq, options)
}

/// Create and return a [`Connection`] and a type inferred [`ConnectAttr`] in tuple
/// using the compile time linked MQ library.
#[inline]
pub fn connect_with<'co, A, H>(options: impl ConnectOption<'co>) -> ResultComp<(Connection<LinkedMq, H>, A)>
where
    A: ConnectAttr<Connection<LinkedMq, H>>,
    H: Threading,
{
    super::connect_lib_as(LinkedMq, options)
}

#[cfg(feature = "mqai")]
impl Bag<Owned, LinkedMq> {
    pub fn new(options: MQCBO) -> ResultComp<Self> {
        Self::new_lib(LinkedMq, options)
    }
}
