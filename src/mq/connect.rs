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
use super::types::QueueManagerName;
use super::MqStruct;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::DerefMut, derive_more::Deref)]
pub struct ConnectionId(pub [sys::MQBYTE; 24]);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, derive_more::DerefMut, derive_more::Deref)]
pub struct ConnTag(pub [sys::MQBYTE; 128]);

/// Associated connection handle and MQ library
pub trait Conn {
    type Lib: Library<MQ: function::Mqi>;
    fn mq(&self) -> &MqFunctions<Self::Lib>;
    fn handle(&self) -> &core::ConnectionHandle;
}

/// A connection to an IBM MQ queue manager
#[derive(Debug)]
pub struct Connection<L: Library<MQ: function::Mqi>, H> {
    handle: core::ConnectionHandle,
    mq: core::MqFunctions<L>,
    _share: PhantomData<H>, // Send and Sync control
}

#[derive(Debug, Clone)]
pub struct ConnectionRef<'conn, L: Library<MQ: function::Mqi>, H> {
    handle: core::ConnectionHandle,
    mq: core::MqFunctions<L>,
    _share: PhantomData<H>,       // Send and Sync control
    _ref: PhantomData<&'conn ()>, // Reference to original connection handle
}

pub struct QueueManager<C: Conn>(pub C);

/// MQCNO parameter used to define the connection
pub type ConnectParam<'a> = MqStruct<'a, sys::MQCNO>;

trait Sealed {}

/// `QueueManagerShare` threading behaviour. Refer to `ShareNone`, `ShareNonBlock` and `ShareBlock`
#[expect(private_bounds)] // Reason: Deliberate implementation of a sealed trait
pub trait HandleShare: Sealed + Clone {
    /// One of the `MQCNO_HANDLE_SHARE_*` MQ constants
    const MQCNO_HANDLE_SHARE: sys::MQLONG;
}

pub trait IntoConnection<C> {
    fn into_connection(self) -> C;
}

impl<C: Conn> IntoConnection<Self> for C {
    fn into_connection(self) -> Self {
        self
    }
}

impl<C: Conn> IntoConnection<C> for QueueManager<C> {
    fn into_connection(self) -> C {
        self.0
    }
}

/// The [`QueueManagerShare`] can only be used in the thread it was created.
/// See the `MQCNO_HANDLE_SHARE_NONE` connection option.
#[derive(Debug, Clone)]
pub struct ShareNone(PhantomData<*const ()>); // !Send + !Sync

/// The [`QueueManagerShare`] can be moved to other threads, but only one thread can use it at any one time.
/// See the `MQCNO_HANDLE_SHARE_NO_BLOCK` connection option.
#[derive(Debug, Clone)]
pub struct ShareNonBlock(PhantomData<*const ()>); // Send + !Sync

/// The [`QueueManagerShare`] can be moved to other threads, and be used by multiple threads concurrently. Blocks when multiple threads call a function.
/// See the `MQCNO_HANDLE_SHARE_BLOCK` connection option.
#[derive(Debug, Clone)]
pub struct ShareBlock; // Send + Sync

impl Sealed for ShareNone {}
impl Sealed for ShareNonBlock {}
impl Sealed for ShareBlock {}
unsafe impl Send for ShareNonBlock {}

impl HandleShare for ShareNone {
    const MQCNO_HANDLE_SHARE: sys::MQLONG = sys::MQCNO_HANDLE_SHARE_NONE;
}

impl HandleShare for ShareBlock {
    const MQCNO_HANDLE_SHARE: sys::MQLONG = sys::MQCNO_HANDLE_SHARE_BLOCK;
}

impl HandleShare for ShareNonBlock {
    const MQCNO_HANDLE_SHARE: sys::MQLONG = sys::MQCNO_HANDLE_SHARE_NO_BLOCK;
}

impl<L: Library<MQ: function::Mqi>, H> Drop for Connection<L, H> {
    fn drop(&mut self) {
        let _ = self.mq.mqdisc(&mut self.handle);
    }
}

impl<L: Library<MQ: function::Mqi>, H: HandleShare, P> MqiValue<P, Self> for Connection<L, H> {
    type Error = Error;

    fn consume<F>(param: &mut P, connect: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut P) -> ResultComp<Self>,
    {
        connect(param)
    }
}

impl<T: From<Connection<L, H>> + Conn, L: Library<MQ: function::Mqi>, H: HandleShare, P> MqiValue<P, Connection<L, H>>
    for QueueManager<T>
{
    type Error = Error;

    fn consume<F>(param: &mut P, connect: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut P) -> ResultComp<Connection<L, H>>,
    {
        connect(param).map_completion(|connection| Self(connection.into()))
    }
}

pub trait ConnectValue<S>: for<'a> MqiValue<ConnectParam<'a>, S, Error = Error> {}
impl<S, T> ConnectValue<S> for T where T: for<'a> MqiValue<ConnectParam<'a>, S, Error = Error> {}
pub trait ConnectAttr<S>: for<'a> MqiAttr<ConnectParam<'a>, S> {}
impl<S, T> ConnectAttr<S> for T where T: for<'a> MqiAttr<ConnectParam<'a>, S> {}

impl<L: Library<MQ: function::Mqi>, H: HandleShare> Connection<L, H> {
    /// Create and return a connection to a queue manager using a specified MQ [`Library`].
    pub fn connect_lib<'co, O>(lib: L, options: O) -> ResultComp<Self>
    where
        O: ConnectOption<'co>,
    {
        Self::connect_lib_as(lib, options)
    }

    /// Create and return a connection to a queue manager using a specified MQ [`Library`] and inferred [`ConnectAttr`].
    pub fn connect_lib_with<'co, O, A>(lib: L, options: O) -> ResultComp<(Self, A)>
    where
        O: ConnectOption<'co>,
        A: ConnectAttr<Self>,
    {
        Self::connect_lib_as(lib, options)
    }

    /// Create a connection to a queue manager using a specified MQ [`Library`] and inferred return value.
    pub(super) fn connect_lib_as<'co, R, O>(lib: L, options: O) -> ResultComp<R>
    where
        R: ConnectValue<Self>,
        O: ConnectOption<'co>,
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
                .map_completion(|handle| Self {
                    mq,
                    handle,
                    _share: PhantomData,
                })
        })
    }
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

    fn handle(&self) -> &ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: function::Mqi>, H> Conn for Rc<Connection<L, H>> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> &ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: function::Mqi>, H> Conn for &Connection<L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        Connection::<L, H>::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        Connection::<L, H>::handle(self)
    }
}

impl<L: Library<MQ: function::Mqi>, H> Conn for Connection<L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> &ConnectionHandle {
        &self.handle
    }
}

impl<'handle, L: Library<MQ: function::Mqi>, H> Conn for ConnectionRef<'handle, L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> &ConnectionHandle {
        &self.handle
    }
}
