use libmqm_sys::link::LinkedMq;
#[cfg(feature = "mqai")]
use {crate::bag, crate::types::MQCBO};

use crate::{
    Connection, connect_lib_as,
    connection::{ConnectAttr, ConnectOption, ConnectValue, ThreadNone, Threading},
    result::ResultComp,
};

/// Connect using the compile-time linked MQ library and the default
/// [`ThreadNone`](crate::connection::ThreadNone) threading mode.
///
/// This is the recommended entry point for applications that do not need to
/// choose a threading mode. Use [`connect`] when the connection must be
/// [`ThreadNoBlock`](crate::connection::ThreadNoBlock) or
/// [`ThreadBlock`](crate::connection::ThreadBlock).
///
/// Pass [`NO_CONNECT_OPTIONS`](crate::connection::NO_CONNECT_OPTIONS) when
/// MQI defaults are sufficient, or pass one option or a tuple of options such
/// as [`ApplName`](crate::types::ApplName) and
/// [`Credentials`](crate::connection::Credentials). For a client connection,
/// combine [`MqServer`](crate::connection::MqServer) with credentials; add
/// [`Tls`](crate::connection::Tls) when the channel requires TLS. See
/// [`ConnectOption`](crate::connection::ConnectOption) for the common option
/// list and a complete example.
///
/// The returned [`ResultComp`] may contain an MQ warning; decide whether to handle it with
/// [`warn_as_error`](crate::result::ResultCompExt::warn_as_error) or
/// [`discard_warning`](crate::result::ResultCompErrExt::discard_warning).
///
/// # Example
///
/// ```no_run
/// use mqi::{connection::NO_CONNECT_OPTIONS, prelude::*};
///
/// let connection = mqi::connect_default(NO_CONNECT_OPTIONS).warn_as_error()?;
/// connection.disconnect().warn_as_error()?;
/// # Ok::<(), mqi::result::Error>(())
/// ```
#[inline]
pub fn connect_default<'co>(options: &impl ConnectOption<'co>) -> ResultComp<Connection<LinkedMq, ThreadNone>> {
    connect::<ThreadNone>(options)
}

/// Create a connection using the compile-time linked MQ library, inferring the
/// returned value from the expected type.
///
/// Use this when the caller's expected type should select a custom
/// [`ConnectValue`] implementation. For the common case of returning a
/// [`Connection`], [`connect`] makes the threading mode explicit and
/// [`connect_default`] selects [`ThreadNone`].
///
/// The options are the same as for [`connect`]: pass a single option or a tuple
/// of options. MQ warnings remain available in the returned [`ResultComp`].
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use mqi::{
///     Connection,
///     connection::{Credentials, ThreadNone},
///     prelude::*,
///     types::ApplName,
/// };
///
/// let username = std::env::var("MQ_USER")?;
/// let password = std::env::var("MQ_PASSWORD")?;
/// let options = (
///     ApplName(mqstr!("my_application")),
///     Credentials::User(&username, password.as_str().into()),
/// );
/// let connection: Connection<_, ThreadNone> =
///     mqi::connect_as(&options).warn_as_error()?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn connect_as<'co, R, H>(options: &impl ConnectOption<'co>) -> ResultComp<R>
where
    R: ConnectValue<Connection<LinkedMq, H>>,
    H: Threading,
{
    connect_lib_as(LinkedMq, options)
}

/// Connect to a queue manager using the compile-time linked MQ library.
///
/// Use [`connect_default`] for the common [`ThreadNone`] mode. This generic
/// function is for choosing an explicit threading mode:
///
/// | Thread type | Connection behavior |
/// | --- | --- |
/// | [`ThreadNone`](crate::connection::ThreadNone) | Use only from the thread that created the connection. |
/// | [`ThreadNoBlock`](crate::connection::ThreadNoBlock) | May move between threads, but concurrent use is not allowed. |
/// | [`ThreadBlock`](crate::connection::ThreadBlock) | May be shared across threads; concurrent MQ calls block. |
///
/// `options` may be a single [`ConnectOption`] or a tuple combining options.
/// Wrap any individual option in `Some(...)` or `None` to enable or omit it
/// based on runtime configuration. `None` skips that option; it does not
/// override a conflicting setting supplied elsewhere in the tuple.
/// Use [`NO_CONNECT_OPTIONS`](crate::connection::NO_CONNECT_OPTIONS) to
/// explicitly request MQI defaults. This function invokes
/// [`MQCONNX`](libmqm_sys::MQCONNX) and returns a [`ResultComp`] so successful
/// MQ warnings can be handled by the caller.
///
/// ## Examples
///
/// ```no_run
/// use mqi::{
///     connection::{Credentials, ThreadBlock},
///     constants,
///     prelude::*,
/// };
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let username = std::env::var("MQ_USER")?;
/// let password = std::env::var("MQ_PASSWORD")?;
/// let options = (
///     constants::MQCNO_RECONNECT_Q_MGR,
///     Credentials::User(&username, password.as_str().into()),
/// );
/// let connection = mqi::connect::<ThreadBlock>(&options).warn_as_error()?;
/// connection.disconnect().warn_as_error()?;
/// # Ok(())
/// # }
/// ```
///
/// Use [`connect_as`] to infer a custom return value or [`connect_with`] to
/// retrieve an additional connection attribute alongside the connection. For a
/// dynamically loaded or custom MQ library, use
/// [`connect_lib`](crate::connect_lib).
///
/// ## Panics
/// This will panic when:
/// * Any MQ structure Version numbers exceed the compiled in MQ client
/// * Any MQ structure Offsets exceed the bounds of an [`MQLONG`](libmqm_sys::MQLONG)
#[inline]
pub fn connect<'co, H>(options: &impl ConnectOption<'co>) -> ResultComp<Connection<LinkedMq, H>>
where
    H: Threading,
{
    connect_lib_as(LinkedMq, options)
}

/// Connect and return an additional value extracted through [`ConnectAttr`].
///
/// The result is `(connection, attribute)`. Use this when an MQ connection
/// attribute such as [`ConnectionId`](crate::connection::ConnectionId) or
/// [`ConnTag`](crate::connection::ConnTag) is needed immediately after connect;
/// otherwise prefer [`connect`] or [`connect_default`].
///
/// Options and threading behavior are the same as for [`connect`]. This
/// function invokes [`MQCONNX`](libmqm_sys::MQCONNX), and its
/// [`ResultComp`] may contain a warning.
///
/// # Example
///
/// ```no_run
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use mqi::{
///     connection::{ConnectionId, Credentials, ThreadNone},
///     prelude::*,
///     types::ApplName,
/// };
///
/// let username = std::env::var("MQ_USER")?;
/// let password = std::env::var("MQ_PASSWORD")?;
/// let options = (
///     ApplName(mqstr!("my_application")),
///     Credentials::User(&username, password.as_str().into()),
/// );
/// let (connection, connection_id) =
///     mqi::connect_with::<ConnectionId, ThreadNone>(&options).warn_as_error()?;
/// let _ = connection_id;
/// connection.disconnect().warn_as_error()?;
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn connect_with<'co, A, H>(options: &impl ConnectOption<'co>) -> ResultComp<(Connection<LinkedMq, H>, A)>
where
    A: ConnectAttr<Connection<LinkedMq, H>>,
    H: Threading,
{
    connect_lib_as(LinkedMq, options)
}

#[cfg(feature = "mqai")]
impl bag::Bag<bag::Owned, LinkedMq> {
    pub fn new(options: MQCBO) -> ResultComp<Self> {
        Self::new_lib(LinkedMq, options)
    }
}
