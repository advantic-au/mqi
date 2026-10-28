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
/// Dropping the connection attempts to disconnect, but cannot return an MQ
/// warning or error. Call [`disconnect`](Self::disconnect) when the outcome
/// needs to be observed.

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
/// This is equivalent of a [`MQHCONN`](mq::MQHCONN) handle. A reference borrows
/// the underlying connection and dropping it does not disconnect. Use
/// [`Connection::connection_ref`] to create a reference whose lifetime is
/// bounded by an owned connection.
#[derive(Debug)]
#[must_use]
pub struct ConnectionRef<'conn, L: Library<MQ: Mqi>, H> {
    /// Connection instance used for dereferencing
    conn: ManuallyDrop<Connection<L, H>>,
    /// Reference to original connectio
    _ref: PhantomData<&'conn ()>,
}

/// Holds either an owned [`Connection`] or a referenced [`ConnectionRef`].
///
/// The `Owned` variant disconnects when dropped. The `Ref` variant does not
/// own the MQ connection and dropping it does not disconnect.
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
    /// Convert the connection into a reference with a caller-selected lifetime.
    ///
    /// This deliberately prevents the connection from disconnecting when the
    /// returned reference is dropped. Use it only when the MQ connection is
    /// intended to remain active for the rest of the process; prefer
    /// [`connection_ref`](Self::connection_ref) for ordinary borrowing.
    pub const fn leak<'a>(self) -> ConnectionRef<'a, L, H> {
        let handle = self.handle;
        // SAFETY: moving mq (Library) to ConnectionRef
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
    /// Create a non-owning reference to the contained connection.
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

/// Create a connection using the supplied MQ [`Library`] and threading mode.
///
/// Use this instead of the linked [`connect`](crate::connect) when loading the
/// MQ library at runtime or supplying a custom library implementation. For a
/// compile-time linked connection with the common
/// [`ThreadNone`](crate::connection::ThreadNone) mode, use
/// [`connect_default`](crate::connect_default).
///
/// `options` may be a single
/// [`ConnectOption`](crate::connection::ConnectOption) or a tuple
/// combining connection settings such as the application name, credentials,
/// TLS, and channel definition. Pass
/// [`NO_CONNECT_OPTIONS`](crate::connection::NO_CONNECT_OPTIONS) for MQI defaults.
///
/// The `H` type controls the connection's thread-safety guarantees; see
/// [`Threading`](crate::connection::Threading). The function invokes
/// [`MQCONNX`](libmqm_sys::MQCONNX) and returns a
/// [`ResultComp`](crate::result::ResultComp) that preserves successful MQ warnings.
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

/// Connect using the supplied MQ [`Library`] and return a connection plus an
/// additional [`ConnectAttr`](crate::connection::ConnectAttr).
///
/// This is the library-parameterized counterpart to
/// [`connect_with`](crate::connect_with). The result is
/// `(connection, attribute)`. Use it when you need an MQ attribute, such as a
/// connection identifier, as well as a connection. See [`connect_lib`] for
/// library selection, options, threading, and warning behavior.
///
/// The function invokes [`MQCONNX`](libmqm_sys::MQCONNX).
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

/// Connect using the supplied MQ [`Library`] and infer the returned value from
/// its expected type.
///
/// This is the general form behind [`connect_lib`] and
/// [`connect_lib_with`](crate::connect_lib_with). Use it when a custom
/// [`ConnectValue`](crate::connection::ConnectValue) determines the returned value; for
/// the usual connection result, prefer [`connect_lib`].
///
/// The function invokes [`MQCONNX`](libmqm_sys::MQCONNX).
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
    /// Disconnect from the queue manager and return the MQ completion status.
    ///
    /// Prefer this over relying on `Drop` when disconnect warnings or errors
    /// need to be handled by the application.
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
    #[cfg(feature = "mock")]
    #[test]
    pub fn connection_either() {
        use super::*;
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
