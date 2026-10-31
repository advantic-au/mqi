use libmqm_sys::Mqi;

use crate::{
    Library,
    result::ResultComp,
    structs,
    types::{self, QueueManagerName},
};

/// Has the ability to return a reference to a [Connection](crate::Connection)
pub trait AsConnection {
    type Lib: Library<MQ: Mqi>;
    type Thread;

    /// Returns a reference to the contained [Connection](crate::Connection)
    fn as_connection(&self) -> &crate::Connection<Self::Lib, Self::Thread>;
}

/// [`MQCNO`](libmqm_sys::MQCNO) parameter used to define the connection
pub type ConnectParam<'a> = structs::MQCNO<'a>;

/// An explicit value for connecting without additional connection options.
pub const NO_CONNECT_OPTIONS: &() = &();

/// [`Connection`](crate::Connection) threading behaviour.
///
/// This must be one of [`ThreadNone`](crate::connection::ThreadNone), [`ThreadNoBlock`](crate::connection::ThreadNoBlock)
/// or [`ThreadBlock`](crate::connection::ThreadBlock). This value will influence the [`Send`] and [`Sync`] traits
/// on the [`Connection`](crate::Connection).
///
/// For more information on multithreading support for MQ connections refer to [thread independent connections].
///
/// [thread independent connections]: https://www.ibm.com/docs/en/SSFKSJ_latest/develop/q025940_.html
pub trait Threading {
    /// One of the `MQCNO_HANDLE_SHARE_*` MQ constants
    const MQCNO_HANDLE_SHARE: types::MQLONG;
}

/// A trait that represents the value of an outcome of an MQ connection call
pub trait ConnectValue<S> {
    /// # Errors
    /// Returns an error if the MQ connection operation fails or the value cannot be extracted.
    fn connect_consume<'a, F>(param: &mut ConnectParam<'a>, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut ConnectParam<'a>) -> ResultComp<S>,
        Self: std::marker::Sized;
}

/// A trait that represents an attribute of an outcome of an MQ connection call
pub trait ConnectAttr<S> {
    /// # Errors
    /// Returns an error if the MQ connection operation fails or the attribute cannot be extracted.
    fn connect_extract<'a, F>(param: &mut ConnectParam<'a>, mqi: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'a>) -> ResultComp<S>,
        Self: std::marker::Sized;
}

/// A connection setting accepted by the [`connect`](crate::connect) functions.
///
/// Implementations are provided for common connection settings, and tuples let
/// you combine them. Users normally pass these values rather than implementing
/// this trait themselves.
///
/// Common options include:
///
/// - [`ApplName`](crate::types::ApplName) to identify the application.
/// - [`Credentials`](crate::connection::Credentials) to provide user credentials.
/// - [`MqServer`](crate::connection::MqServer) to select a client channel and server.
/// - [`Tls`](crate::connection::Tls) or [`CipherSpec`](crate::types::CipherSpec) to configure TLS.
/// - [`QueueManagerName`](crate::types::QueueManagerName) to select a queue manager.
/// - [`Binding`](crate::connection::Binding) to select local or client binding.
/// - [`Ccdt`](crate::connection::Ccdt) to select a client channel definition table URL.
/// - [`MQCNO`](libmqm_sys::MQCNO) flags and other low-level MQ structures for
///   settings not covered by the higher-level options.
///
/// Use [`NO_CONNECT_OPTIONS`] when no additional settings are needed.
///
/// Wrap an option in `Option<T>` to decide at runtime whether to apply it:
/// `Some(value)` applies the option and `None` leaves it unapplied. This is
/// useful when assembling a tuple of options from runtime configuration. The
/// `Option` wrapper does not clear or override a setting applied by another
/// option in the tuple; avoid conflicting settings unless their precedence is
/// intentional. The same pattern is supported by [`OpenOption`](crate::open::OpenOption).
///
/// # Example
///
/// ```no_run
/// use mqi::{
///     connection::{Credentials, MqServer, NO_CONNECT_OPTIONS, Tls},
///     prelude::*,
///     types::{ApplName, CipherSpec, KeyRepo, QueueName},
/// };
///
/// #[cfg(feature = "link")]
/// fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let username = std::env::var("MQ_USER")?;
///     let password = std::env::var("MQ_PASSWORD")?;
///     let use_tls = std::env::var("MQ_USE_TLS").is_ok();
///     let key_repo = KeyRepo(mqstr!("tls.kdb"));
///     let cipher = CipherSpec(mqstr!("TLS_AES_128_GCM_SHA256"));
///     let tls = use_tls.then(|| Tls::new(&key_repo, None, &cipher));
///     let options = (
///         ApplName(mqstr!("my_application")),
///         Credentials::User(&username, password.as_str().into()),
///         MqServer::try_from("DEV.APP.SVRCONN/TCP/localhost(1414)")?,
///         tls,
///     );
///     let connection = mqi::connect_default(&options).warn_as_error()?;
///     connection.put_message(&QueueName(mqstr!("DEV.QUEUE.1")), &(), "Hello")
///         .warn_as_error()?;
///     connection.disconnect().warn_as_error()?;
///
///     let connection = mqi::connect_default(NO_CONNECT_OPTIONS).warn_as_error()?;
///     connection.disconnect().warn_as_error()?;
///     Ok(())
/// }
/// #[cfg(not(feature = "link"))]
/// fn main() {}
/// ```
///
/// Tuple options are applied from right to left. If multiple options set the
/// same MQ field, the leftmost option is applied last and takes precedence for
/// that field. Prefer a single option for each setting unless you intentionally
/// need to override it; options that modify several related fields can still
/// interact.
///
/// This trait manipulates the parameters to the
/// [`MQCONNX`](libmqm_sys::MQCONNX) function.
#[expect(unused_variables)]
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `ConnectOption` so it can't be used as an argument for MQI connect"
)]
/// ## Safety
/// This trait can directly manipulate the [`MQCNO`](structs::MQCNO) structure which is used by [`MQCONNX`](libmqm_sys::MQCONNX).
/// Incorrect values in the [`MQCNO`](structs::MQCNO) structure can lead to undefined behaviour.
///
/// Implementations of [`ConnectOption`] must ensure that pointers and offsets contained in the structure point to active data.
pub unsafe trait ConnectOption<'a> {
    /// Returns the queue manager name to connect to, or `None` to use the default queue manager name.
    #[inline]
    fn queue_manager_name(&self) -> Option<&QueueManagerName> {
        None
    }

    /// Applies the type to to the structures contained in [`ConnectStructs`].
    ///
    /// Returns a mask indicating which structures are used by the type.
    #[inline]
    fn apply_param(&self, structs: &mut ConnectStructs<'a>) -> ConnectStructFlags {
        CONNECT_HAS_CNO
    }
}

/// A collection of MQ structures used by MQ at connection time
#[derive(Debug, Clone)]
pub struct ConnectStructs<'ptr> {
    pub cno: structs::MQCNO<'ptr>,
    pub sco: structs::MQSCO<'ptr>,
    pub csp: structs::MQCSP<'ptr>,
    pub cd: structs::MQCD<'ptr>,
    #[cfg(feature = "mqc_9_3_0_0")]
    pub bno: structs::MQBNO,
}

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    derive_more::BitAnd,
    derive_more::BitAndAssign,
    derive_more::BitOr,
    derive_more::BitXorAssign,
)]
pub struct ConnectStructFlags(usize);

pub const CONNECT_HAS_NONE: ConnectStructFlags = ConnectStructFlags(0b00000);

/// A [`MQCNO`](libmqm_sys::MQCNO) structure is required for the connection option
pub const CONNECT_HAS_CNO: ConnectStructFlags = ConnectStructFlags(0b00000);

/// A [`MQSCO`](libmqm_sys::MQSCO) structure is required for the connection option
pub const CONNECT_HAS_SCO: ConnectStructFlags = ConnectStructFlags(0b00010);

/// A [`MQCD`](libmqm_sys::MQCD) structure is required for the connection option
pub const CONNECT_HAS_CD: ConnectStructFlags = ConnectStructFlags(0b00100);

/// A [`MQSCSP`](libmqm_sys::MQCSP) structure is required for the connection option
pub const CONNECT_HAS_CSP: ConnectStructFlags = ConnectStructFlags(0b01000);

#[cfg(feature = "mqc_9_3_0_0")]
/// A [`MQBNO`](libmqm_sys::MQBNO) structure is required for the connection option
pub const CONNECT_HAS_BNO: ConnectStructFlags = ConnectStructFlags(0b10000);
