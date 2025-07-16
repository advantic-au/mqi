use std::{
    fmt::Debug,
    marker::PhantomData,
    mem::ManuallyDrop,
    ops::{Deref, DerefMut},
    ptr::drop_in_place,
    rc::Rc,
    sync::Arc,
};

use libmqm_sys::{self as mq, Mqi};

use super::option;
use crate::{Library, MqFunctions, handle::ConnectionHandle, prelude::*, result::ResultComp, types};

/// A connection to an IBM MQ queue manager
#[derive(Debug)]
pub struct Connection<L: Library<MQ: Mqi>, H> {
    handle: ConnectionHandle,
    mq: MqFunctions<L>,
    _share: PhantomData<H>, // Send and Sync control
}

#[derive(Debug)]
pub struct ConnectionRef<'conn, L: Library<MQ: Mqi>, H> {
    conn: ManuallyDrop<Connection<L, H>>,
    _ref: PhantomData<&'conn ()>, // Reference to original connection handle
}

impl<L: Library<MQ: Mqi> + Clone, H> Clone for ConnectionRef<'_, L, H> {
    fn clone(&self) -> Self {
        ConnectionRef::from_parts(self.handle, self.mq.clone())
    }
}

impl<L: Library<MQ: Mqi>, H> Deref for ConnectionRef<'_, L, H> {
    type Target = Connection<L, H>;

    fn deref(&self) -> &Self::Target {
        &self.conn
    }
}

impl<L: Library<MQ: Mqi>, H> DerefMut for ConnectionRef<'_, L, H> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.conn
    }
}

impl<L, H> Connection<L, H>
where
    L: Library<MQ: Mqi> + Clone,
{
    #[inline]
    pub fn connection_ref(&self) -> ConnectionRef<'_, L, H> {
        ConnectionRef::from_parts(self.handle, self.mq.clone())
    }

    pub fn leak<'a>(self) -> ConnectionRef<'a, L, H> {
        let handle = self.handle;
        let mq = self.mq.clone();
        let _ = ManuallyDrop::new(self);
        ConnectionRef::from_parts(handle, mq)
    }

    #[inline]
    pub fn library(&self) -> L {
        self.mq.0.clone()
    }
}

impl<L: Library<MQ: Mqi>, H> Drop for ConnectionRef<'_, L, H> {
    fn drop(&mut self) {
        let ConnectionRef { conn, .. } = self;
        unsafe {
            drop_in_place(&raw mut conn.mq);
        }
    }
}

impl<L, H> ConnectionRef<'_, L, H>
where
    L: Library<MQ: Mqi>,
{
    pub const fn from_parts(handle: ConnectionHandle, mq: MqFunctions<L>) -> Self {
        Self {
            conn: ManuallyDrop::new(Connection {
                handle,
                mq,
                _share: PhantomData,
            }),
            _ref: PhantomData,
        }
    }
}

/// The [`Connection`] can only be used in the thread it was created.
/// See the `MQCNO_HANDLE_SHARE_NONE` connection option.
#[derive(Debug, Clone, Copy)]
pub struct ThreadNone(PhantomData<*const ()>); // !Send + !Sync

/// The [`Connection`] can be moved between threads, but only one thread can use it at any one time.
/// See the `MQCNO_HANDLE_SHARE_NO_BLOCK` connection option.
#[derive(Debug, Clone, Copy)]
pub struct ThreadNoBlock(PhantomData<*const ()>); // Send + !Sync

/// The [`Connection`] can be moved between threads, and be used by multiple threads concurrently. Blocks when multiple threads call a function.
/// See the `MQCNO_HANDLE_SHARE_BLOCK` connection option.
#[derive(Debug, Clone, Copy)]
pub struct ThreadBlock; // Send + Sync

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

/// Create a connection to a queue manager using a [`Library`] returning a [`Connection`].
///
/// The connection parameters are provided using a [`ConnectOption`](option::ConnectOption). Multiple [`ConnectOption`](option::ConnectOption) can
/// be supplied using tuples of varying length.
///
/// This function uses the [`MQCONNX`](libmqm_sys::MQCONNX) verb.
///
/// ## Panics
/// This will panic when:
/// * Any MQ structure Version exceeds the compiled MQ client
/// * Any MQ structure Offset exceedd the bounds of an [`MQLONG`](types::MQLONG)
pub fn connect_lib<'co, H, L>(lib: L, options: &impl option::ConnectOption<'co>) -> ResultComp<Connection<L, H>>
where
    H: option::Threading,
    L: Library<MQ: Mqi>,
{
    connect_lib_as(lib, options)
}

/// Create a connection to a queue manager using a [`Library`] returning a ([`Connection`], [impl `ConnectAttr`](option::ConnectAttr)) tuple.
///
/// The connection parameters are provided using a [`ConnectOption`](option::ConnectOption). Multiple [`ConnectOption`](option::ConnectOption) can
/// be supplied using tuples of varying length.
///
/// This function uses the [`MQCONNX`](libmqm_sys::MQCONNX) verb.
///
/// ## Panics
/// This will panic when:
/// * Any MQ structure Version exceeds the compiled MQ client
/// * Any MQ structure Offset exceedd the bounds of an [`MQLONG`](types::MQLONG)
pub fn connect_lib_with<'co, A, H, L>(lib: L, options: &impl option::ConnectOption<'co>) -> ResultComp<(Connection<L, H>, A)>
where
    A: option::ConnectAttr<Connection<L, H>>,
    H: option::Threading,
    L: Library<MQ: Mqi>,
{
    connect_lib_as(lib, options)
}

/// Create a connection to a queue manager using a [`Library`] returning a usually inferred [`ConnectValue`](option::ConnectValue).
///
/// The connection parameters are provided using a [`ConnectOption`](option::ConnectOption). Multiple [`ConnectOption`](option::ConnectOption) can
/// be supplied using tuples of varying length.
///
/// This function uses the [`MQCONNX`](libmqm_sys::MQCONNX) verb.
///
/// ## Panics
/// This will panic when:
/// * Any MQ structure Version exceeds the compiled MQ client
/// * Any MQ structure Offset exceedd the bounds of an [`MQLONG`](types::MQLONG)
pub fn connect_lib_as<'co, R, H, L>(lib: L, options: &impl option::ConnectOption<'co>) -> ResultComp<R>
where
    R: option::ConnectValue<Connection<L, H>>,
    H: option::Threading,
    L: Library<MQ: Mqi>,
{
    let qm_name = options.queue_manager_name().copied();

    let mut structs = option::ConnectStructs::default();
    let struct_mask = options.apply_param(&mut structs);
    assert!(structs.cno.Version <= mq::MQCNO_CURRENT_VERSION);

    let cno_ptr = &raw const structs.cno;
    #[cfg(feature = "mqc_9_3_0_0")]
    if struct_mask & option::CONNECT_HAS_BNO != option::CONNECT_HAS_NONE {
        assert!(structs.bno.Version <= mq::MQBNO_CURRENT_VERSION);
        structs.cno.set_min_version(mq::MQCNO_VERSION_8);
        structs.cno.BalanceParmsOffset = unsafe { (&raw const structs.bno).byte_offset_from(cno_ptr) }
            .try_into()
            .expect("MQBNO offset from MQCNO should convert to MQLONG");
    }

    if struct_mask & option::CONNECT_HAS_CD != option::CONNECT_HAS_NONE {
        assert!(structs.cd.Version <= mq::MQCD_CURRENT_VERSION);
        structs.cno.set_min_version(mq::MQCNO_VERSION_2);
        structs.cno.ClientConnOffset = unsafe { (&raw const structs.cd).byte_offset_from(cno_ptr) }
            .try_into()
            .expect("MQCD offset from MQCNO should convert to MQLONG");
    }

    if struct_mask & option::CONNECT_HAS_SCO != option::CONNECT_HAS_NONE {
        assert!(structs.sco.Version <= mq::MQSCO_CURRENT_VERSION);
        structs.cno.set_min_version(mq::MQCNO_VERSION_4);
        structs.cno.SSLConfigOffset = unsafe { (&raw const structs.sco).byte_offset_from(cno_ptr) }
            .try_into()
            .expect("MQSCO offset from MQCNO should convert to MQLONG");
    }

    if struct_mask & option::CONNECT_HAS_CSP != option::CONNECT_HAS_NONE {
        assert!(structs.csp.Version <= mq::MQCSP_CURRENT_VERSION);
        structs.cno.set_min_version(mq::MQCNO_VERSION_5);
        structs.cno.SecurityParmsOffset = unsafe { (&raw const structs.csp).byte_offset_from(cno_ptr) }
            .try_into()
            .expect("MQCSP offset from MQCNO should convert to MQLONG");
    }

    R::connect_consume(&mut structs.cno, |cno| {
        cno.Options |= H::MQCNO_HANDLE_SHARE;
        let mq = MqFunctions(lib);
        let qm_default = types::QueueManagerName::default(); // TODO: change to constant
        let qm = qm_name.as_ref().map_or(&qm_default, |qm| qm);

        // SAFETY: Implementors of ConnectOption must ensure MQCNO and associated structures are correctly populated
        unsafe {
            mq.mqconnx(qm, cno).map_completion(|handle| Connection {
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

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for Rc<Connection<L, H>> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        self.deref().mq()
    }

    fn handle(&self) -> ConnectionHandle {
        self.deref().handle()
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for &Connection<L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        Connection::<L, H>::mq(self)
    }

    fn handle(&self) -> ConnectionHandle {
        Connection::<L, H>::handle(self)
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for Connection<L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> ConnectionHandle {
        self.handle
    }
}

impl<L: Library<MQ: Mqi>, H> crate::Conn for ConnectionRef<'_, L, H> {
    type Lib = L;

    fn mq(&self) -> &MqFunctions<Self::Lib> {
        &self.mq
    }

    fn handle(&self) -> ConnectionHandle {
        self.handle
    }
}
