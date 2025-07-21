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
use crate::{Library, MqFunctions, connection::AsConnection, handle::ConnectionHandle, prelude::*, result::ResultComp, types};

/// A connection to an IBM MQ queue manager
///
/// This is equivalent of a [`MQHCONN`](mq::MQHCONN) handle, with additional associated MQ library.

#[derive(Debug)]
pub struct Connection<L: Library<MQ: Mqi>, H> {
    /// Core connection handle
    pub(crate) handle: ConnectionHandle,
    /// MQ functions associatedc with the connection
    pub(crate) mq: MqFunctions<L>,
    _share: PhantomData<H>, // Send and Sync control
}

/// A logical reference to a [`Connection`]
///
/// This is equivalent of a [`MQHCONN`](mq::MQHCONN) handle. Lifetime of this reference is linked to a
/// real [`Connection`].
#[derive(Debug)]
#[must_use]
pub struct ConnectionRef<'conn, L: Library<MQ: Mqi>, H> {
    /// Connection instance used for dereferencing
    conn: ManuallyDrop<Connection<L, H>>,
    /// Reference to original connectio
    _ref: PhantomData<&'conn ()>,
}

/// Holds a [`Connection`] or a referenced connection [`ConnectionRef`]
#[derive(Debug)]
#[must_use]
pub enum ConnectionEither<'conn, L: Library<MQ: Mqi>, H> {
    Owned(Connection<L, H>),
    Ref(ConnectionRef<'conn, L, H>),
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
    /// Create a referenced connection
    pub fn connection_ref(&self) -> ConnectionRef<'_, L, H> {
        ConnectionRef::from_parts(self.handle, self.mq.clone())
    }

    /// Clone the library associated with the connection
    #[inline]
    pub fn library(&self) -> L {
        self.mq.0.clone()
    }
}

impl<L, H> Connection<L, H>
where
    L: Library<MQ: Mqi>,
{
    /// Leak the connection to a static reference
    ///
    /// This is typically used to have a Connection that is active for the entire lifetime of an
    /// application, without closing the connection.
    pub const fn leak<'a>(self) -> ConnectionRef<'a, L, H> {
        let handle = self.handle;
        let mq = unsafe { std::ptr::read(&raw const self.mq) };
        let _ = ManuallyDrop::new(self);
        ConnectionRef::from_parts(handle, mq)
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

impl<L: Library<MQ: Mqi>, H> AsConnection for ConnectionEither<'_, L, H> {
    type Lib = L;
    type Thread = H;

    fn as_connection(&self) -> &crate::Connection<Self::Lib, Self::Thread> {
        match self {
            ConnectionEither::Owned(connection) => connection.as_connection(),
            ConnectionEither::Ref(connection_ref) => connection_ref.as_connection(),
        }
    }
}

impl<'a, L: Library<MQ: Mqi>, H> ConnectionEither<'a, L, H> {
    pub fn connection_ref<'b: 'a>(&'b self) -> ConnectionRef<'a, L, H>
    where
        L: Clone,
    {
        match self {
            ConnectionEither::Owned(connection) => connection.connection_ref(),
            ConnectionEither::Ref(connection_ref) => connection_ref.clone(),
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

impl<L: Library<MQ: Mqi>, H> option::AsConnection for Connection<L, H> {
    type Lib = L;
    type Thread = H;

    fn as_connection(&self) -> &crate::Connection<Self::Lib, Self::Thread> {
        self
    }
}

impl<L: Library<MQ: Mqi>, H> option::AsConnection for ConnectionRef<'_, L, H> {
    type Lib = L;
    type Thread = H;

    fn as_connection(&self) -> &crate::Connection<Self::Lib, Self::Thread> {
        &self.conn
    }
}

impl<T: option::AsConnection<Lib = L, Thread = H>, L: Library<MQ: Mqi>, H> option::AsConnection for Rc<T> {
    type Lib = L;
    type Thread = H;

    fn as_connection(&self) -> &crate::Connection<Self::Lib, Self::Thread> {
        self.deref().as_connection()
    }
}

impl<T: option::AsConnection<Lib = L, Thread = H>, L: Library<MQ: Mqi>, H> option::AsConnection for Arc<T> {
    type Lib = L;
    type Thread = H;

    fn as_connection(&self) -> &crate::Connection<Self::Lib, Self::Thread> {
        self.deref().as_connection()
    }
}

impl<T: option::AsConnection<Lib = L, Thread = H>, L: Library<MQ: Mqi>, H> option::AsConnection for &T {
    type Lib = L;
    type Thread = H;

    fn as_connection(&self) -> &crate::Connection<Self::Lib, Self::Thread> {
        (*self).as_connection()
    }
}

#[cfg(test)]
pub mod test {

    use super::*;

    #[cfg(feature = "mock")]
    #[test]
    pub fn connection_either() {
        use crate::test::mock;

        let connection = mock::connect_ok(|_| {});
        let handle = connection.handle;
        let cr = connection.connection_ref();
        let ce_r = ConnectionEither::Ref(cr.clone());

        // Test ConnectionEither::Ref
        assert_eq!(ce_r.connection_ref().handle, handle);
        assert_eq!(ce_r.as_connection().handle, handle);

        // Test ConnectionEither::Owned
        let connection = mock::connect_ok(|_| {});
        let ce_o = ConnectionEither::Owned(connection);
        assert_eq!(ce_o.connection_ref().handle, handle);
        assert_eq!(ce_o.as_connection().handle, handle);
    }

}