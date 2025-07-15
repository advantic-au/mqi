use libmqm_sys::Mqi;

use crate::{
    Library, MqFunctions,
    handle::ConnectionHandle,
    result::ResultComp,
    structs,
    types::{self, QueueManagerName},
};

/// Associated connection handle and MQ library
pub trait Conn {
    type Lib: Library<MQ: Mqi>;
    type Thread;

    fn mq(&self) -> &MqFunctions<Self::Lib>;
    fn handle(&self) -> ConnectionHandle;
}

/// [`MQCNO`](libmqm_sys::MQCNO) parameter used to define the connection
pub type ConnectParam<'a> = structs::MQCNO<'a>;

/// [`Connection`](crate::Connection) threading behaviour.
///
/// This must be one of [`ThreadNone`](crate::connection::ThreadNone), [`ThreadNoBlock`](crate::connection::ThreadBlock)
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
    fn connect_consume<'a, F>(param: &mut ConnectParam<'a>, mqi: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut ConnectParam<'a>) -> ResultComp<S>,
        Self: std::marker::Sized;
}

/// A trait that represents an attribute of an outcome of an MQ connection call
pub trait ConnectAttr<S> {
    fn connect_extract<'a, F>(param: &mut ConnectParam<'a>, mqi: F) -> ResultComp<(Self, S)>
    where
        F: FnOnce(&mut ConnectParam<'a>) -> ResultComp<S>,
        Self: std::marker::Sized;
}

/// A trait that manipulates the parameters to the [`MQCONNX`](`::libmqm_sys::MQCONNX`) function
#[expect(unused_variables)]
#[diagnostic::on_unimplemented(
    message = "{Self} does not implement `ConnectOption` so it can't be used as an argument for MQI connect"
)]
/// # Safety
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
