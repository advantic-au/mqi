use super::option;
use std::{fmt::Debug, marker::PhantomData, ops::Deref, rc::Rc, sync::Arc};

use libmqm_sys::{self as mq, Mqi};

// #[cfg(feature = "link")]
// pub use crate::link::*;
use crate::{ConnectionHandle, Library, MqFunctions, ResultComp, prelude::*, types};

/// A connection to an IBM MQ queue manager
#[derive(Debug)]
pub struct Connection<L: Library<MQ: Mqi>, H> {
    handle: ConnectionHandle,
    mq: MqFunctions<L>,
    _share: PhantomData<H>, // Send and Sync control
}

#[derive(Debug, Clone, Copy)]
pub struct ConnectionRef<'conn, L: Library<MQ: Mqi>, H> {
    handle: ConnectionHandle,
    mq: MqFunctions<L>,
    _share: PhantomData<H>,       // Send and Sync control
    _ref: PhantomData<&'conn ()>, // Reference to original connection handle
}

impl<L, H> Connection<L, H>
where
    L: Library<MQ: Mqi> + Clone,
{
    #[inline]
    pub fn connection_ref(&self) -> ConnectionRef<'_, L, H> {
        ConnectionRef::from_parts(self.handle, self.mq.clone())
    }

    #[inline]
    pub fn library(&self) -> L
    where
        L: Clone,
    {
        self.mq.0.clone()
    }
}

impl<L, H> ConnectionRef<'_, L, H>
where
    L: Library<MQ: Mqi>,
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

impl option::Sealed for ThreadNone {}
impl option::Sealed for ThreadNoBlock {}
impl option::Sealed for ThreadBlock {}
unsafe impl Send for ThreadNoBlock {}

impl option::Threading for ThreadNone {
    const MQCNO_HANDLE_SHARE: types::MQLONG = mq::MQCNO_HANDLE_SHARE_NONE;
}

impl option::Threading for ThreadBlock {
    const MQCNO_HANDLE_SHARE: types::MQLONG = mq::MQCNO_HANDLE_SHARE_BLOCK;
}

impl option::Threading for ThreadNoBlock {
    const MQCNO_HANDLE_SHARE: types::MQLONG = mq::MQCNO_HANDLE_SHARE_NO_BLOCK;
}

impl<L: Library<MQ: Mqi>, H> Drop for Connection<L, H> {
    fn drop(&mut self) {
        let _ = self.mq.mqdisc(&mut self.handle);
    }
}

impl<L: Library<MQ: Mqi>, H: option::Threading> option::ConnectValue<Self> for Connection<L, H> {
    #[inline]
    fn connect_consume<'a, F>(param: &mut option::ConnectParam<'a>, connect: F) -> ResultComp<Self>
    where
        F: FnOnce(&mut option::ConnectParam<'a>) -> ResultComp<Self>,
    {
        connect(param)
    }
}

/// Create and return a [`Connection`] to a queue manager using a specified MQ [`Library`].
pub fn connect_lib<'co, H, L>(lib: L, options: &impl option::ConnectOption<'co>) -> ResultComp<Connection<L, H>>
where
    H: option::Threading,
    L: Library<MQ: Mqi>,
{
    connect_lib_as(lib, options)
}

/// Create and return a [`Connection`] to a queue manager using a specified MQ [`Library`] and inferred [`ConnectAttr`](option::ConnectAttr).
pub fn connect_lib_with<'co, A, H, L>(lib: L, options: &impl option::ConnectOption<'co>) -> ResultComp<(Connection<L, H>, A)>
where
    A: option::ConnectAttr<Connection<L, H>>,
    H: option::Threading,
    L: Library<MQ: Mqi>,
{
    connect_lib_as(lib, options)
}

/// Create a [`Connection`] to a queue manager using a specified MQ [`Library`] and inferred return value.
pub fn connect_lib_as<'co, R, H, L>(lib: L, options: &impl option::ConnectOption<'co>) -> ResultComp<R>
where
    R: option::ConnectValue<Connection<L, H>>,
    H: option::Threading,
    L: Library<MQ: Mqi>,
{
    let qm_name = options.queue_manager_name().copied();

    let mut structs = option::ConnectStructs::default();
    let struct_mask = options.apply_param(&mut structs);

    let cno_ptr = &raw const structs.cno;
    #[cfg(feature = "mqc_9_3_0_0")]
    if struct_mask & option::CONNECT_HAS_BNO != option::CONNECT_HAS_NONE {
        structs.cno.set_min_version(mq::MQCNO_VERSION_8);
        structs.cno.BalanceParmsOffset = unsafe { (&raw const structs.bno).byte_offset_from(cno_ptr) }
            .try_into()
            .expect("MQBNO offset from MQCNO should convert to i32");
    }

    if struct_mask & option::CONNECT_HAS_CD != option::CONNECT_HAS_NONE {
        structs.cno.set_min_version(mq::MQCNO_VERSION_2);
        structs.cno.ClientConnOffset = unsafe { (&raw const structs.cd).byte_offset_from(cno_ptr) }
            .try_into()
            .expect("MQCD offset from MQCNO should convert to i32");
    }

    if struct_mask & option::CONNECT_HAS_SCO != option::CONNECT_HAS_NONE {
        structs.cno.set_min_version(mq::MQCNO_VERSION_4);
        structs.cno.SSLConfigOffset = unsafe { (&raw const structs.sco).byte_offset_from(cno_ptr) }
            .try_into()
            .expect("MQSCO offset from MQCNO should convert to i32");
    }

    if struct_mask & option::CONNECT_HAS_CSP != option::CONNECT_HAS_NONE {
        {
            structs.cno.set_min_version(mq::MQCNO_VERSION_5);
            structs.cno.SecurityParmsOffset = unsafe { (&raw const structs.csp).byte_offset_from(cno_ptr) }
                .try_into()
                .expect("MQCSP offset from MQCNO should convert to i32");
        };
    }

    R::connect_consume(&mut structs.cno, |param| {
        param.Options |= H::MQCNO_HANDLE_SHARE;
        let mq = MqFunctions(lib);
        let qm_default = types::QueueManagerName::default(); // TODO: change to constant
        let qm = qm_name.as_ref().map_or(&qm_default, |qm| qm);

        // SAFETY: Implementors of ConnectOption must ensure MQCNO and associated structures are correctly populated
        unsafe {
            mq.mqconnx(qm, param).map_completion(|handle| Connection {
                mq,
                handle,
                _share: PhantomData,
            })
        }
    })
}

impl<L: Library<MQ: Mqi>, H> Connection<L, H> {
    pub fn disconnect(self) -> ResultComp<()> {
        let mut s = self;
        s.mq.mqdisc(&mut s.handle)
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for Arc<Connection<L, H>> {
    type Lib = L;
    type Thread = H;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for Rc<Connection<L, H>> {
    type Lib = L;
    type Thread = H;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for &Connection<L, H> {
    type Lib = L;
    type Thread = H;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        Connection::<L, H>::mq(self)
    }

    fn handle(&self) -> ConnectionHandle {
        Connection::<L, H>::handle(self)
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for Connection<L, H> {
    type Lib = L;
    type Thread = H;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> ConnectionHandle {
        self.handle
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for ConnectionRef<'_, L, H> {
    type Lib = L;
    type Thread = H;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> ConnectionHandle {
        self.handle
    }
}
