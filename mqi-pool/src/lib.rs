//! A synchronous [`r2d2`] connection pool adapter for [`mqi`].
//!
//! This is an early prototype. It pools MQ connections using
//! [`ThreadNoBlock`](mqi::connection::ThreadNoBlock), so a checked-out
//! connection can move between threads but is exclusively owned by one
//! borrower at a time. The MQ library type is generic, allowing linked,
//! dynamically loaded, or custom `mqi::Library` implementations.
//!
//! # Example
//!
//! ```no_run
//! use mqi::{
//!     connection::{Credentials, ThreadNoBlock},
//!     prelude::*,
//!     types::ApplName,
//! };
//! use mqi_pool::MqiConnectionManager;
//! use r2d2::Pool;
//! use libmqm_sys::link::LinkedMq;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let username = std::env::var("MQ_USER")?;
//! let password = std::env::var("MQ_PASSWORD")?;
//! let manager = MqiConnectionManager::<LinkedMq, _>::new(move || {
//!     let options = (
//!         ApplName(mqstr!("mqi_pool_example")),
//!         Credentials::User(&username, password.as_str().into()),
//!     );
//!     mqi::connect::<ThreadNoBlock>(&options)
//! });
//! let pool = Pool::builder().max_size(8).build(manager)?;
//! let connection = pool.get()?;
//! connection.put_message(
//!     &mqi::types::QueueName(mqstr!("DEV.QUEUE.1")),
//!     &(),
//!     "Hello from a pooled connection",
//! ).warn_as_error()?;
//! # Ok(())
//! # }
//! ```
//!
//! MQ calls are synchronous and block the thread. This pool does not make MQI
//! operations asynchronous.
//!
//! # Transactions
//!
//! ```no_run
//! use mqi::{prelude::*, types::QueueName};
//! use mqi_pool::{MqConnection, MqiConnectionManager};
//! use r2d2::Pool;
//! use libmqm_sys::link::LinkedMq;
//!
//! # fn run<F>(pool: &Pool<MqiConnectionManager<LinkedMq, F>>) -> Result<(), Box<dyn std::error::Error>>
//! # where
//! #     F: Fn() -> mqi::result::ResultComp<MqConnection<LinkedMq>> + Send + Sync + 'static,
//! # {
//! let mut connection = pool.get()?;
//! let mut transaction = connection.transaction()?;
//! transaction
//!     .put_message(
//!         &QueueName(mqstr!("DEV.QUEUE.1")),
//!         &mqi::constants::MQPMO_SYNCPOINT,
//!         "Transactional message",
//!     )
//!     .warn_as_error()?;
//! transaction.commit().warn_as_error()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Transaction safety
//!
//! Start units of work with [`PooledConnection::transaction`] and finish them
//! using [`Transaction::commit`] or [`Transaction::backout`]. Dropping an
//! unfinished transaction attempts a backout. The guard owns one
//! [`mqi::Syncpoint`] for its entire lifetime; it uses that instance to commit,
//! back out, or perform drop cleanup. If commit or backout fails or returns an
//! MQ warning, the connection is marked broken and discarded when returned to
//! the pool. Since dropping a transaction cannot report cleanup failures,
//! prefer explicit commit or backout; even an unused guard may be discarded
//! if MQ reports that no unit of work was active.
//!
//! The connection API remains available through dereferencing, so callers
//! must not perform syncpoint work outside the transaction guard. The pool
//! cannot detect such work or automatically determine whether an MQ connection
//! is live. Call [`PooledConnection::mark_broken`] after an operation leaves
//! the connection unusable or its state uncertain.

use std::{
    cell::Cell,
    error::Error as StdError,
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use libmqm_sys::Mqi;
use mqi::{
    Connection, ConnectionRef, Library, Syncpoint,
    connection::{AsConnection, ConnectionEither, ThreadNoBlock},
    result::{Completion, ResultComp, ResultCompErrExt},
    types::MQRC,
};
use r2d2::ManageConnection;

/// The connection type managed by this pool adapter.
pub type MqConnection<L> = Connection<L, ThreadNoBlock>;
type PooledSyncpoint<'connection, L> = Syncpoint<ConnectionRef<'connection, L, ThreadNoBlock>>;

/// Errors reported while creating or checking a pooled MQ connection.
#[derive(Debug)]
pub enum PoolError {
    /// Creating an MQ connection failed.
    Connect(mqi::result::Error),
    /// MQCONNX succeeded with a warning; the pool rejects the connection
    /// because it cannot return that warning to the borrower.
    ConnectWarning(MQRC, &'static str),
    /// MQCONNX reported that the connection is already owned elsewhere.
    AlreadyConnected,
    /// The connection was explicitly marked broken or has an unfinished
    /// transaction whose guard was forgotten.
    BrokenConnection,
    /// A transaction is already active on this lease.
    TransactionAlreadyActive,
}

impl fmt::Display for PoolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => write!(formatter, "could not connect to IBM MQ: {error}"),
            Self::ConnectWarning(reason, verb) => {
                write!(formatter, "IBM MQ connection returned a warning from {verb}: {reason}")
            }
            Self::AlreadyConnected => formatter.write_str("IBM MQ connection is already owned elsewhere"),
            Self::BrokenConnection => formatter.write_str("IBM MQ connection is marked broken"),
            Self::TransactionAlreadyActive => formatter.write_str("an MQ transaction is already active"),
        }
    }
}

impl StdError for PoolError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::ConnectWarning(..) | Self::AlreadyConnected | Self::BrokenConnection | Self::TransactionAlreadyActive => None,
        }
    }
}

/// Factory-backed manager for MQ connections using the library type `L`.
///
/// The closure is called each time the pool creates a connection. Capture
/// owned configuration in it and construct any borrowed [`mqi::ConnectOption`]
/// values inside the closure. Use [`mqi::connect_lib`] to provide a dynamic or
/// custom library implementation. `L` must be cloneable because mqi connection
/// references retain a clone of the library, and sendable because r2d2 may
/// move connections between threads.
pub struct MqiConnectionManager<L, F> {
    connect: F,
    library: PhantomData<fn() -> L>,
}

impl<L, F> MqiConnectionManager<L, F> {
    /// Create a manager from a connection factory.
    #[must_use]
    pub const fn new(connect: F) -> Self {
        Self {
            connect,
            library: PhantomData,
        }
    }
}

impl<L, F> ManageConnection for MqiConnectionManager<L, F>
where
    L: Library<MQ: Mqi> + Clone + Send + Sync + 'static,
    MqConnection<L>: Send + 'static,
    F: Fn() -> ResultComp<MqConnection<L>> + Send + Sync + 'static,
{
    type Connection = PooledConnection<L>;
    type Error = PoolError;

    fn connect(&self) -> Result<Self::Connection, Self::Error> {
        let Completion(connection, warning) = (self.connect)()
            .already_connected_ref::<L, ThreadNoBlock>()
            .map_err(PoolError::Connect)?;
        let connection = match connection {
            ConnectionEither::Owned(connection) => connection,
            ConnectionEither::Ref(_) => return Err(PoolError::AlreadyConnected),
        };
        if let Some((reason, verb)) = warning {
            drop(connection);
            return Err(PoolError::ConnectWarning(reason, verb));
        }

        Ok(PooledConnection {
            connection,
            broken: Cell::new(false),
            transaction_active: Cell::new(false),
        })
    }

    fn is_valid(&self, connection: &mut Self::Connection) -> Result<(), Self::Error> {
        if connection.broken.get() || connection.transaction_active.get() {
            Err(PoolError::BrokenConnection)
        } else {
            Ok(())
        }
    }

    fn has_broken(&self, connection: &mut Self::Connection) -> bool {
        connection.broken.get() || connection.transaction_active.get()
    }
}

/// An exclusively checked-out MQ connection.
///
/// The underlying [`MqConnection`] is available through [`Deref`] and
/// [`DerefMut`]. The pool only knows that a connection is unusable when it is
/// explicitly marked broken or a transaction guard is left unfinished.
pub struct PooledConnection<L: Library<MQ: Mqi> + Clone> {
    connection: MqConnection<L>,
    broken: Cell<bool>,
    transaction_active: Cell<bool>,
}

impl<L: Library<MQ: Mqi> + Clone> PooledConnection<L> {
    /// Mark this connection unusable so it is discarded instead of reused.
    ///
    /// Call this after a fatal MQ error or whenever the connection's state is
    /// uncertain. It cannot be reset.
    pub fn mark_broken(&mut self) {
        self.broken.set(true);
    }

    /// Return whether this connection has been marked unusable.
    #[must_use]
    pub const fn is_broken(&self) -> bool {
        self.broken.get()
    }

    /// Start managing an MQ unit of work on this checked-out connection.
    ///
    /// Operations performed on the returned guard participate in MQ syncpoint
    /// processing when passed MQ syncpoint options. This does not call
    /// `MQBEGIN`; use [`mqi::Syncpoint::begin`] separately for units of work
    /// involving an external resource manager.
    ///
    /// An unfinished guard attempts a backout when dropped. If that backout
    /// fails or returns a warning, the pooled connection is marked broken.
    /// An unused guard can receive a warning because no unit of work was
    /// active; finish guards explicitly when possible.
    pub fn transaction(&mut self) -> Result<Transaction<'_, L>, PoolError> {
        if self.broken.get() || self.transaction_active.get() {
            return Err(if self.transaction_active.get() {
                PoolError::TransactionAlreadyActive
            } else {
                PoolError::BrokenConnection
            });
        }

        self.transaction_active.set(true);
        Ok(Transaction {
            syncpoint: Some(Syncpoint::new(self.connection.connection_ref())),
            broken: &self.broken,
            transaction_active: &self.transaction_active,
        })
    }
}

impl<L: Library<MQ: Mqi> + Clone> Deref for PooledConnection<L> {
    type Target = MqConnection<L>;

    fn deref(&self) -> &Self::Target {
        &self.connection
    }
}

impl<L: Library<MQ: Mqi> + Clone> DerefMut for PooledConnection<L> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.connection
    }
}

/// A transaction guard that exclusively borrows a pooled connection.
///
/// Use the dereferenced connection for MQ operations. Explicitly call
/// [`commit`](Self::commit) or [`backout`](Self::backout) to observe the MQ
/// result. Dropping an active guard makes a best-effort backout.
#[must_use = "dropping an unfinished transaction attempts a backout"]
pub struct Transaction<'connection, L: Library<MQ: Mqi> + Clone> {
    syncpoint: Option<PooledSyncpoint<'connection, L>>,
    broken: &'connection Cell<bool>,
    transaction_active: &'connection Cell<bool>,
}

impl<L: Library<MQ: Mqi> + Clone> Transaction<'_, L> {
    /// Commit the current MQ unit of work.
    ///
    /// If MQ reports an error or warning, the pooled connection is marked
    /// broken because the pool cannot safely assume its state.
    pub fn commit(mut self) -> ResultComp<()> {
        let syncpoint = self
            .syncpoint
            .take()
            .expect("transaction guard always owns its syncpoint until completion");
        self.transaction_active.set(false);
        self.broken.set(true);
        let result = syncpoint.commit().map(|completion| completion.map(|_| ()));
        self.finish(result)
    }

    /// Back out the current MQ unit of work.
    ///
    /// If MQ reports an error or warning, the pooled connection is marked
    /// broken because the pool cannot safely assume its state.
    pub fn backout(mut self) -> ResultComp<()> {
        let syncpoint = self
            .syncpoint
            .take()
            .expect("transaction guard always owns its syncpoint until completion");
        self.transaction_active.set(false);
        self.broken.set(true);
        let result = syncpoint.backout().map(|completion| completion.map(|_| ()));
        self.finish(result)
    }

    fn finish(self, result: ResultComp<()>) -> ResultComp<()> {
        match result {
            Ok(completion) => {
                self.broken.set(completion.warning().is_some());
                Ok(completion.map(|_| ()))
            }
            Err(error) => {
                self.broken.set(true);
                Err(error)
            }
        }
    }
}

impl<L: Library<MQ: Mqi> + Clone> Deref for Transaction<'_, L> {
    type Target = MqConnection<L>;

    fn deref(&self) -> &Self::Target {
        self.syncpoint
            .as_ref()
            .expect("active transaction guard owns its syncpoint")
            .as_ref()
    }
}

impl<L: Library<MQ: Mqi> + Clone> AsConnection for Transaction<'_, L> {
    type Lib = L;
    type Thread = ThreadNoBlock;

    fn as_connection(&self) -> &Connection<Self::Lib, Self::Thread> {
        self.syncpoint
            .as_ref()
            .expect("active transaction guard owns its syncpoint")
            .as_ref()
    }
}

impl<L: Library<MQ: Mqi> + Clone> Drop for Transaction<'_, L> {
    fn drop(&mut self) {
        if let Some(syncpoint) = self.syncpoint.take() {
            self.broken.set(true);
            self.transaction_active.set(false);
            let result = syncpoint.backout();
            if matches!(result, Ok(ref completion) if completion.warning().is_none()) {
                self.broken.set(false);
            }
        } else {
            self.transaction_active.set(false);
        }
    }
}
