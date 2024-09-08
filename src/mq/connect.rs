use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

use libmqm_sys::function;

use crate::core::{self, ConnectionHandle, Library, MQFunctions};
use crate::{sys, Error, MqiAttr, MqiValue, ResultCompErrExt as _};
use crate::ResultComp;

use super::connect_options::{self, ConnectOption, ConnectStructs};
use super::types::QueueManagerName;
use super::MqStruct;

pub struct ConnectionId(pub [sys::MQBYTE; 24]);
pub struct ConnTag(pub [sys::MQBYTE; 128]);

pub trait Conn {
    type Lib: Library<MQ: function::MQI>;
    fn mq(&self) -> &MQFunctions<Self::Lib>;
    fn handle(&self) -> &core::ConnectionHandle;
}

/// A connection to an IBM MQ queue manager
#[derive(Debug)]
pub struct QueueManagerShare<'cb, L: Library<MQ: function::MQI>, H> {
    handle: core::ConnectionHandle,
    mq: core::MQFunctions<L>,
    _share: PhantomData<H>,         // Send and Sync control
    _ref: PhantomData<&'cb mut ()>, // Connection may refer to callback function
}

/// Thread movable `QueueManagerShare`
pub type QueueManager<'cb, L> = QueueManagerShare<'cb, L, ShareBlock>;

pub type ConnectParam<'a> = MqStruct<'a, sys::MQCNO>;

trait Sealed {}

/// `QueueManagerShare` threading behaviour. Refer to `ShareNone`, `ShareNonBlock` and `ShareBlock`
#[expect(private_bounds)] // Reason: Deliberate implementation of a sealed trait
pub trait HandleShare: Sealed {
    /// One of the `MQCNO_HANDLE_SHARE_*` MQ constants
    const MQCNO_HANDLE_SHARE: sys::MQLONG;
}

#[expect(dead_code)]
/// The `Connection` can only be used in the thread it was created.
/// See the `MQCNO_HANDLE_SHARE_NONE` connection option.
#[derive(Debug)]
pub struct ShareNone(*const ()); // !Send + !Sync

#[expect(dead_code)]
/// The `Connection` can be moved to other threads, but only one thread can use it at any one time.
/// See the `MQCNO_HANDLE_SHARE_NO_BLOCK` connection option.
#[derive(Debug)]
pub struct ShareNonBlock(*const ()); // Send + !Sync

/// The `Connection` can be moved to other threads, and be used by multiple threads concurrently. Blocks when multiple threads call a function.
/// See the `MQCNO_HANDLE_SHARE_BLOCK` connection option.
#[derive(Debug)]
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

impl<L: Library<MQ: function::MQI>, H> Drop for QueueManagerShare<'_, L, H> {
    fn drop(&mut self) {
        let _ = self.mq.mqdisc(&mut self.handle);
    }
}

impl<L: Library<MQ: function::MQI>, H: HandleShare, P> MqiValue<P, Self> for QueueManagerShare<'_, L, H> {
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

impl<L: Library<MQ: function::MQI>, H: HandleShare> QueueManagerShare<'_, L, H> {
    pub fn connect_lib<'co, O>(lib: L, options: O) -> ResultComp<Self>
    where
        O: ConnectOption<'co>,
    {
        Self::connect_lib_as(lib, options)
    }

    pub fn connect_lib_with<'co, O, A>(lib: L, options: O) -> ResultComp<(Self, A)>
    where
        O: ConnectOption<'co>,
        A: ConnectAttr<Self>,
    {
        Self::connect_lib_as(lib, options)
    }

    /// Create a connection to a queue manager using the provided `qm_name` and the `MQCNO` builder
    pub(super) fn connect_lib_as<'co, R, O>(lib: L, options: O) -> ResultComp<R>
    where
        R: ConnectValue<Self>,
        O: ConnectOption<'co>,
    {
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

        let qm_name = options.queue_manager_name();

        R::consume(&mut structs.cno, |param| {
            param.Options |= H::MQCNO_HANDLE_SHARE;
            let mq = core::MQFunctions(lib);
            mq.mqconnx(qm_name.unwrap_or(&QueueManagerName::default()), param)
                .map_completion(|handle| Self {
                    mq,
                    handle,
                    _share: PhantomData,
                    _ref: PhantomData,
                })
        })
    }
}

impl<'cb, L: Library<MQ: function::MQI>, H> QueueManagerShare<'cb, L, H> {
    #[must_use]
    pub const fn mq(&self) -> &MQFunctions<L> {
        &self.mq
    }

    #[must_use]
    pub const fn handle(&self) -> &core::ConnectionHandle {
        &self.handle
    }

    pub fn disconnect(self) -> ResultComp<()> {
        let mut s = self;
        s.mq.mqdisc(&mut s.handle)
    }

    pub fn syncpoint(&mut self) -> Syncpoint<'_, 'cb, L, H> {
        Syncpoint {
            connection: self,
            state: SyncpointState::Open,
        }
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for Arc<QueueManagerShare<'_, L, H>> {
    type Lib = L;

    fn mq(&self) -> &MQFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> &ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for QueueManagerShare<'_, L, H> {
    type Lib = L;

    fn mq(&self) -> &MQFunctions<Self::Lib> {
        Self::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        Self::handle(self)
    }
}

impl<L: Library<MQ: function::MQI>, H> Conn for &QueueManagerShare<'_, L, H> {
    type Lib = L;

    fn mq(&self) -> &MQFunctions<Self::Lib> {
        QueueManagerShare::<L, H>::mq(self)
    }

    fn handle(&self) -> &ConnectionHandle {
        QueueManagerShare::<L, H>::handle(self)
    }
}

#[derive(Debug)]
enum SyncpointState {
    Open,
    Committed,
    Backout,
}

#[must_use]
pub struct Syncpoint<'connection, 'cb, L: Library<MQ: function::MQI>, H> {
    state: SyncpointState,
    connection: &'connection mut QueueManagerShare<'cb, L, H>,
}

impl<L: Library<MQ: function::MQI>, H> Syncpoint<'_, '_, L, H> {
    pub fn commit(self) -> ResultComp<()> {
        let result = self.mq.mqcmit(self.handle());
        let mut self_mut = self;
        self_mut.state = SyncpointState::Committed;
        result
    }

    pub fn backout(self) -> ResultComp<()> {
        let result = self.mq.mqback(self.handle());
        let mut self_mut = self;
        self_mut.state = SyncpointState::Backout;
        result
    }
}

impl<L: Library<MQ: function::MQI>, H> Drop for Syncpoint<'_, '_, L, H> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if matches!(self.state, SyncpointState::Open) {
            let _ = self.mq.mqback(self.handle());
        }
    }
}

impl<'cb, L: Library<MQ: function::MQI>, H> Deref for Syncpoint<'_, 'cb, L, H> {
    type Target = QueueManagerShare<'cb, L, H>;

    fn deref(&self) -> &Self::Target {
        self.connection
    }
}
