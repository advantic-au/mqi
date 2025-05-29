use libmqm_sys::link::LinkedMq;
#[cfg(feature = "mqai")]
use {
    crate::types::MQCBO,
    crate::{Bag, Owned},
};

use super::{ConnectAttr, ConnectValue, Connection, Threading, connect_options::ConnectOption};
use crate::ResultComp;

/// Create a connection to a queue manager using the compile time linked MQ library
/// and type inferred [`ConnectValue`].
#[inline]
pub fn connect_as<'co, R, H>(options: &impl ConnectOption<'co>) -> ResultComp<R>
where
    R: ConnectValue<Connection<LinkedMq, H>>,
    H: Threading,
{
    super::connect_lib_as(LinkedMq, options)
}

/// Create a connection to a queue manager using the compile time linked MQ library.
///
/// The connection parameters are controlled using a [`ConnectOption`]. Multiple [`ConnectOption`] can
/// be supplied using tuples of varying length.
///
/// The [`Threading`] type parameter controls the threaded capability of the connection.
///
/// This uses the [`MQCONNX`](libmqm_sys::Mqi::MQCONNX) function.
///
/// # Examples
///
/// ```no_run
/// use mqi::prelude::*;
/// use mqi::{ThreadNone, connect_options::Credentials, constants};
///
/// // Connect to the default queue manager with the provided credentials and MQCNO_RECONNECT_Q_MGR
/// let connection = mqi::connect::<ThreadNone>(&(
///     constants::MQCNO_RECONNECT_Q_MGR, Credentials::User("app", "app".into())
/// ))?;
///
/// // connection is wrapped in a Completion. Discard the completion with a `discard_warning`
/// let connection = connection.discard_warning();
///
/// # Ok::<(), mqi::Error>(())
/// ```
///
/// See also [`connect_as`] and [`connect_with`] for creating connections with additional
/// return attribute. For connections using dynamically loaded or custom implementation of the
/// MQ library refer to [`connect_lib`](crate::connect_lib).
#[inline]
pub fn connect<'co, H>(options: &impl ConnectOption<'co>) -> ResultComp<Connection<LinkedMq, H>>
where
    H: Threading,
{
    super::connect_lib_as(LinkedMq, options)
}

/// Create a connection to a queue manager and return an implementation of [`ConnectAttr`] in tuple
/// using the compile time linked MQ library.
///
/// Refer to [`connect`] for parameter details.
///
/// This uses the [`MQCONNX`](libmqm_sys::Mqi::MQCONNX) function.
///
/// Common [`ConnectAttr`] that can be returned include [`ConnTag`](crate::ConnTag) and [`ConnectionId`](crate::ConnectionId).
#[inline]
pub fn connect_with<'co, A, H>(options: &impl ConnectOption<'co>) -> ResultComp<(Connection<LinkedMq, H>, A)>
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
