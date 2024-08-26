use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::Deref;

use libmqm_sys::function;

use crate::core::{self, Library, MQFunctions};
use crate::{sys, Error, ResultCompErrExt as _};
use crate::ResultComp;

use super::connect_options::{self, ConnectOption};
use super::types::QueueManagerName;
use super::{ConsumeValue2, MqStruct};

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
#[allow(private_bounds)] // Reason: Deliberate implementation of a sealed trait
pub trait HandleShare: Sealed {
    /// One of the `MQCNO_HANDLE_SHARE_*` MQ constants
    const MQCNO_HANDLE_SHARE: sys::MQLONG;
}

#[allow(dead_code)]
/// The `Connection` can only be used in the thread it was created.
/// See the `MQCNO_HANDLE_SHARE_NONE` connection option.
#[derive(Debug)]
pub struct ShareNone(*const ()); // !Send + !Sync

#[allow(dead_code)]
/// The `Connection` can be moved to other threads, but only one thread can use it at any one time.
/// See the `MQCNO_HANDLE_SHARE_NO_BLOCK` connection option.
#[derive(Debug)]
pub struct ShareNonBlock(*const ()); // Send + !Sync

#[allow(dead_code)]
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

// impl<L: Library<MQ: function::MQI>, H: HandleShare> MqiValue<Self> for QueueManagerShare<'_, L, H> {
//     type Param<'a> = ConnectParam<'a>;

//     fn from_mqi<F: FnOnce(&mut Self::Param<'_>) -> ResultComp<Self>>(
//         param: &mut Self::Param<'_>,
//         connect: F,
//     ) -> ResultComp<Self> {
//         connect(param)
//     }
// }

impl<L: Library<MQ: function::MQI>, H: HandleShare, P> ConsumeValue2<P, Self> for QueueManagerShare<'_, L, H> {
    type Error = Error;

    fn consume<F>(param: &mut P, connect: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut P) -> ResultComp<Self> {
        connect(param)
    }
}


pub trait QueueManagerValue<S>: for<'a> ConsumeValue2<ConnectParam<'a>, S, Error = Error> {}

// impl<T, A: for<'a> MqiValue<T, Param<'a> = ConnectParam<'a>>> QueueManagerValue<T> for A {}

impl<L: Library<MQ: function::MQI>, H: HandleShare> QueueManagerShare<'_, L, H> {
    /// Create a connection to a queue manager using the provided `qm_name` and the `MQCNO` builder
    pub fn connect_lib<'co, R, O>(lib: L, qm_name: Option<&QueueManagerName>, options: &O) -> ResultComp<R>
    where
        R: QueueManagerValue<Self>,
        O: ConnectOption<'co>,
    {
        let mut cno = MqStruct::default();
        let mut sco = MqStruct::default();
        let mut cd = MqStruct::default();
        let mut bno = MqStruct::default();
        let mut csp = MqStruct::default();

        options.apply_cno(&mut cno);
        if const { O::STRUCTS & connect_options::HAS_BNO != 0 } {
            options.apply_bno(&mut bno);
            cno.attach_bno(&bno);
        }

        if const { O::STRUCTS & connect_options::HAS_CD != 0 } {
            options.apply_cd(&mut cd);
            cno.attach_cd(&cd);
        }

        if const { O::STRUCTS & connect_options::HAS_SCO != 0 } {
            options.apply_sco(&mut sco);
            cno.attach_sco(&sco);
        }

        if const { O::STRUCTS & connect_options::HAS_CSP != 0 } {
            options.apply_csp(&mut csp);
            cno.attach_csp(&csp);
        }    

        R::consume(&mut cno, |param| {
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
