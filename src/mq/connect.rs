use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use libmqm_sys::function;

use crate::core::{self, ConnectionHandle, Library, MqFunctions};
use crate::{sys, prelude::*, Error, MqiAttr, MqiValue};
use crate::ResultComp;

use super::connect_options::{self, ConnectOption, ConnectStructs};
use super::types::{Identifier, QueueManagerName};
use super::MqStruct;

#[cfg(feature = "link")]
pub use super::link::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Deref, derive_more::Display)]
pub struct ConnectionId(pub Identifier<24>);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::Deref)]
pub struct ConnTag(pub [sys::MQBYTE; sys::MQ_CONN_TAG_LENGTH]);

/// Associated connection handle and MQ library
pub trait Conn {
    type Lib: Library<MQ: function::Mqi>;
    fn mq(&self) -> &MqFunctions<Self::Lib>;
    fn handle(&self) -> core::ConnectionHandle;
}

/// A connection to an IBM MQ queue manager
#[derive(Debug)]
pub struct Connection<L: Library<MQ: function::Mqi>, H> {
    handle: core::ConnectionHandle,
    mq: core::MqFunctions<L>,
    _share: PhantomData<H>, // Send and Sync control
}

#[derive(Debug, Clone, Copy)]
pub struct ConnectionRef<'conn, L: Library<MQ: function::Mqi>, H> {
    handle: core::ConnectionHandle,
    mq: core::MqFunctions<L>,
    _share: PhantomData<H>,       // Send and Sync control
    _ref: PhantomData<&'conn ()>, // Reference to original connection handle
}

/// MQCNO parameter used to define the connection
pub type ConnectParam<'a> = MqStruct<'a, sys::MQCNO>;

trait Sealed {}

/// [`Connection`] threading behaviour. This must be one of [`ThreadNone`], [`ThreadNoBlock`] or [`ThreadBlock`].
/// This value will influence the [`Send`] and [`Sync`] traits on the [`Connection`].
///
/// For more information on multithreading support for MQ connections refer to [thread independent connections].
///
/// [thread independent connections]: https://www.ibm.com/docs/en/ibm-mq/latest?topic=call-shared-thread-independent-connections-mqconnx
#[expect(private_bounds, reason = "sealed trait pattern")]
pub trait Threading: Sealed {
    /// One of the `MQCNO_HANDLE_SHARE_*` MQ constants
    const MQCNO_HANDLE_SHARE: sys::MQLONG;
}

impl<L, H> Connection<L, H>
where
    L: Library<MQ: function::Mqi> + Clone,
{
    #[inline]
    pub fn connection_ref(&self) -> ConnectionRef<L, H> {
        ConnectionRef::from_parts(self.handle, self.mq.clone())
    }
}

impl<L, H> ConnectionRef<'_, L, H>
where
    L: Library<MQ: function::Mqi>,
{
    pub const fn from_parts(handle: ConnectionHandle, mq: MqFunctions<L>) -> Self {
        Self {
            handle,
            mq,
            _share: PhantomData,
            _ref: PhantomData,
        }
    }
}

/// The [`Connection`] can only be used in the thread it was created.
/// See the `MQCNO_HANDLE_SHARE_NONE` connection option.
#[derive(Debug, Clone, Copy)]
pub struct ThreadNone(PhantomData<*const ()>); // !Send + !Sync

/// The [`Connection`] can be moved to other threads, but only one thread can use it at any one time.
/// See the `MQCNO_HANDLE_SHARE_NO_BLOCK` connection option.
#[derive(Debug, Clone, Copy)]
pub struct ThreadNoBlock(PhantomData<*const ()>); // Send + !Sync

/// The [`Connection`] can be moved to other threads, and be used by multiple threads concurrently. Blocks when multiple threads call a function.
/// See the `MQCNO_HANDLE_SHARE_BLOCK` connection option.
#[derive(Debug, Clone, Copy)]
pub struct ThreadBlock; // Send + Sync

impl Sealed for ThreadNone {}
impl Sealed for ThreadNoBlock {}
impl Sealed for ThreadBlock {}
unsafe impl Send for ThreadNoBlock {}

impl Threading for ThreadNone {
    const MQCNO_HANDLE_SHARE: sys::MQLONG = sys::MQCNO_HANDLE_SHARE_NONE;
}

impl Threading for ThreadBlock {
    const MQCNO_HANDLE_SHARE: sys::MQLONG = sys::MQCNO_HANDLE_SHARE_BLOCK;
}

impl Threading for ThreadNoBlock {
    const MQCNO_HANDLE_SHARE: sys::MQLONG = sys::MQCNO_HANDLE_SHARE_NO_BLOCK;
}

impl<L: Library<MQ: function::Mqi>, H> Drop for Connection<L, H> {
    fn drop(&mut self) {
        let _ = self.mq.mqdisc(&mut self.handle);
    }
}

impl<L: Library<MQ: function::Mqi>, H: Threading, P> MqiValue<P, Self> for Connection<L, H> {
    type Error = Error;

    fn consume<F>(param: &mut P, connect: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut P) -> ResultComp<Self>,
    {
        connect(param)
    }
}

pub trait ConnectValue<S>: for<'a> MqiValue<ConnectParam<'a>, S, Error = Error> {}
impl<S, T> ConnectValue<S> for T where T: for<'a> MqiValue<ConnectParam<'a>, S, Error = Error> {}
pub trait ConnectAttr<S>: for<'a> MqiAttr<ConnectParam<'a>, S> {}
impl<S, T> ConnectAttr<S> for T where T: for<'a> MqiAttr<ConnectParam<'a>, S> {}

/// Create and return a [`Connection`] to a queue manager using a specified MQ [`Library`].
pub fn connect_lib<'co, H, L>(lib: L, options: impl ConnectOption<'co>) -> ResultComp<Connection<L, H>>
where
    H: Threading,
    L: Library<MQ: function::Mqi>,
{
    connect_lib_as(lib, options)
}

/// Create and return a [`Connection`] to a queue manager using a specified MQ [`Library`] and inferred [`ConnectAttr`].
pub fn connect_lib_with<'co, A, H, L>(lib: L, options: impl ConnectOption<'co>) -> ResultComp<(Connection<L, H>, A)>
where
    A: ConnectAttr<Connection<L, H>>,
    H: Threading,
    L: Library<MQ: function::Mqi>,
{
    connect_lib_as(lib, options)
}

/// Create a [`Connection`] to a queue manager using a specified MQ [`Library`] and inferred return value.
pub(super) fn connect_lib_as<'co, R, H, L>(lib: L, options: impl ConnectOption<'co>) -> ResultComp<R>
where
    R: ConnectValue<Connection<L, H>>,
    H: Threading,
    L: Library<MQ: function::Mqi>,
{
    let qm_name = options.queue_manager_name().copied();

    let mut structs = ConnectStructs::default();
    let struct_mask = options.apply_param(&mut structs);

    if struct_mask & connect_options::HAS_BNO != 0 {
        structs.cno.attach_bno(&structs.bno);
    }

    if struct_mask & connect_options::HAS_CD != 0 {
        structs.cno.attach_cd(&structs.cd);
    }

    if struct_mask & connect_options::HAS_SCO != 0 {
        structs.cno.attach_sco(&structs.sco);
    }

    if struct_mask & connect_options::HAS_CSP != 0 {
        structs.cno.attach_csp(&structs.csp);
    }

    R::consume(&mut structs.cno, |param| {
        param.Options |= H::MQCNO_HANDLE_SHARE;
        let mq = core::MqFunctions(lib);
        mq.mqconnx(qm_name.as_ref().unwrap_or(&QueueManagerName::default()), param)
            .map_completion(|handle| Connection {
                mq,
                handle,
                _share: PhantomData,
            })
    })
}

impl<L: Library<MQ: function::Mqi>, H> Connection<L, H> {
    pub fn disconnect(self) -> ResultComp<()> {
        let mut s = self;
        s.mq.mqdisc(&mut s.handle)
    }
}

impl<L: Library<MQ: function::Mqi>, H> Conn for Arc<Connection<L, H>> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: function::Mqi>, H> Conn for Rc<Connection<L, H>> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: function::Mqi>, H> Conn for &Connection<L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        Connection::<L, H>::mq(self)
    }

    fn handle(&self) -> ConnectionHandle {
        Connection::<L, H>::handle(self)
    }
}

impl<L: Library<MQ: function::Mqi>, H> Conn for Connection<L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> ConnectionHandle {
        self.handle
    }
}

impl<'handle, L: Library<MQ: function::Mqi>, H> Conn for ConnectionRef<'handle, L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> ConnectionHandle {
        self.handle
    }
}
