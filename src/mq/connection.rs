use std::fmt::Debug;
use std::marker::PhantomData;
use std::{fmt::Display, ops::Deref};

use crate::core::{self, Library, MQFunctions};
use crate::{sys, MqStr, ResultCompErrExt as _, StructBuilder};
use crate::{QMName, ResultComp};

type ConnectionId = [sys::MQBYTE; 24];

/// A connection to an IBM MQ queue manager
#[derive(Debug)]
pub struct ConnectionShare<L: Library, H> {
    handle: core::ConnectionHandle,
    mq: core::MQFunctions<L>,
    id: ConnectionId,
    tag: Option<String>,
    _mark: PhantomData<H>, // Send and Sync control
}

/// Thread movable `ConnectionShare`
pub type Connection<L> = ConnectionShare<L, ShareNonBlock>;

trait Sealed {}

/// `Connection` threading behaviour. Refer to `ShareNone`, `ShareNonBlock` and `ShareBlock`
#[allow(private_bounds)] // Reason: Deliberate implementation of a sealed trait
pub trait HandleShare: Sealed {
    /// One of the `MQCNO_HANDLE_SHARE_*` MQ constants
    const MQCNO_HANDLE_SHARE: sys::MQLONG;
}

#[allow(dead_code)]
/// The `Connection` can only be used in the thread it was created.
/// See the `MQCNO_HANDLE_SHARE_NONE` connection option.
pub struct ShareNone(*const ()); // !Send + !Sync

#[allow(dead_code)]
/// The `Connection` can be moved to other threads, but only one thread can use it at any one time.
/// See the `MQCNO_HANDLE_SHARE_NO_BLOCK` connection option.
pub struct ShareNonBlock(*const ()); // Send + !Sync

#[allow(dead_code)]
/// The `Connection` can be moved to other threads, and be used by multiple threads concurrently. Blocks when multiple threads call a function.
/// See the `MQCNO_HANDLE_SHARE_BLOCK` connection option.
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

impl<L: Library, H> Drop for ConnectionShare<L, H> {
    fn drop(&mut self) {
        let _ = self.mq.mqdisc(&mut self.handle);
    }
}

impl<L: Library, H> Display for ConnectionShare<L, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ID(", self.handle)?;
        for byte in self.id {
            write!(f, "{byte:02X}")?;
        }
        write!(f, ")")?;
        if let Some(tag) = &self.tag {
            write!(f, " TAG({tag})")?;
        }
        Ok(())
    }
}

impl<L: Library, H: HandleShare> ConnectionShare<L, H> {
    /// Create a connection to a queue manager using the provided `qm_name` and the `MQCNO` builder
    pub fn new_lib(lib: L, qm_name: Option<&QMName>, builder: &impl StructBuilder<sys::MQCNO>) -> ResultComp<Self> {
        let mut cno_build = builder.build();
        let cno = &mut cno_build;
        cno.Options |= H::MQCNO_HANDLE_SHARE;

        let mq = core::MQFunctions(lib);
        mq.mqconnx(qm_name.unwrap_or(&MqStr::default()), cno)
            .map_completion(|handle| Self {
                mq,
                handle,
                id: cno.ConnectionId,
                tag: Some(
                    String::from_utf8_lossy(&cno.ConnTag)
                        .trim_end_matches(char::from(0))
                        .to_owned(),
                )
                .filter(|t| !t.is_empty()),
                _mark: PhantomData,
            })
    }
}

impl<L: Library, H> ConnectionShare<L, H> {
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn id(&self) -> &ConnectionId {
        &self.id
    }

    #[must_use]
    pub const fn mq(&self) -> &MQFunctions<L> {
        &self.mq
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    #[must_use]
    pub const fn handle(&self) -> &core::ConnectionHandle {
        &self.handle
    }

    pub fn disconnect(self) -> ResultComp<()> {
        let mut s = self;
        s.mq.mqdisc(&mut s.handle)
    }

    pub fn syncpoint(&mut self) -> Syncpoint<'_, L, H> {
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
pub struct Syncpoint<'connection, L: Library, H> {
    state: SyncpointState,
    connection: &'connection mut ConnectionShare<L, H>,
}

impl<L: Library, H> Syncpoint<'_, L, H> {
    pub fn commit(mut self) -> ResultComp<()> {
        let result = self.mq.mqcmit(self.handle());
        self.state = SyncpointState::Committed;
        result
    }

    pub fn backout(mut self) -> ResultComp<()> {
        let result = self.mq.mqback(self.handle());
        self.state = SyncpointState::Backout;
        result
    }
}

impl<L: Library, H> Drop for Syncpoint<'_, L, H> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if matches!(self.state, SyncpointState::Open) {
            let _ = self.mq.mqback(self.handle());
        }
    }
}

impl<L: Library, H> Deref for Syncpoint<'_, L, H> {
    type Target = ConnectionShare<L, H>;

    fn deref(&self) -> &Self::Target {
        self.connection
    }
}
