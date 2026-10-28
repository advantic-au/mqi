use std::mem::ManuallyDrop;

use libmqm_default as default;

use crate::{
    connection::AsConnection,
    result::{ResultComp, ResultCompErrExt},
    structs,
    types::MQBO,
};

/// Manage an MQ unit of work on a connection.
///
/// Use [`new`](Self::new) to coordinate MQI operations marked with
/// `MQGMO_SYNCPOINT` or `MQPMO_SYNCPOINT`; it does not call `MQBEGIN`.
/// [`begin`](Self::begin) explicitly calls `MQBEGIN` for a unit of work that
/// involves an external resource manager. Commit or back out explicitly to
/// observe the MQ completion status. Dropping an unfinished value attempts a
/// backout, but ignores any warning or error from that attempt.
#[derive(Debug)]
#[must_use]
pub struct Syncpoint<C: AsConnection> {
    connection: ManuallyDrop<C>,
}

impl<C: AsConnection> Syncpoint<C> {
    /// Manage the current MQ unit of work without calling `MQBEGIN`.
    ///
    /// MQ operations join the unit of work when called with the appropriate
    /// syncpoint options, such as `MQGMO_SYNCPOINT` for a get and
    /// `MQPMO_SYNCPOINT` for a put.
    pub const fn new(connection: C) -> Self {
        Self {
            connection: ManuallyDrop::new(connection),
        }
    }

    /// Begins a unit of work that is coordinated by the queue manager, and that can involve external resource managers.
    ///
    /// This function uses the [`MQBEGIN`](libmqm_sys::MQBEGIN) verb.
    pub fn begin(connection: C, mqbo: MQBO) -> ResultComp<Self> {
        let mut bo = structs::MQBO::new(libmqm_sys::MQBO {
            Options: mqbo.0,
            ..default::MQBO_DEFAULT
        });
        let conn = connection.as_connection();
        conn.mq
            .mqbegin(conn.handle, Some(&mut bo))
            .map_completion(|()| Self::new(connection))
    }

    /// This function uses the [`MQCMIT`](libmqm_sys::MQCMIT) verb.
    pub fn commit(self) -> ResultComp<C> {
        let mut self_mut = self;
        let conn = self_mut.connection.as_connection();
        let result = conn.mq.mqcmit(conn.handle);
        let wrapped = unsafe { ManuallyDrop::take(&mut self_mut.connection) };
        let _ = ManuallyDrop::new(self_mut); // Suppress default drop
        result.map_completion(|()| wrapped)
    }

    /// This function uses the [`MQBACK`](libmqm_sys::MQBACK) verb.
    pub fn backout(self) -> ResultComp<C> {
        let mut self_mut = self;
        let conn = self_mut.connection.as_connection();
        let result = conn.mq.mqback(conn.handle);
        let wrapped = unsafe { ManuallyDrop::take(&mut self_mut.connection) };
        let _ = ManuallyDrop::new(self_mut); // Suppress default drop
        result.map_completion(|()| wrapped)
    }
}

impl<C: AsConnection> AsConnection for Syncpoint<C> {
    type Lib = C::Lib;
    type Thread = C::Thread;

    fn as_connection(&self) -> &crate::Connection<C::Lib, C::Thread> {
        self.connection.as_connection()
    }
}

impl<C: AsConnection> AsRef<crate::Connection<C::Lib, C::Thread>> for Syncpoint<C> {
    fn as_ref(&self) -> &crate::Connection<C::Lib, C::Thread> {
        self.connection.as_connection()
    }
}

impl<C: AsConnection> Drop for Syncpoint<C> {
    fn drop(&mut self) {
        let conn = self.connection.as_connection();
        let _ = conn.mq.mqback(conn.handle);
        unsafe { ManuallyDrop::drop(&mut self.connection) };
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #[cfg(feature = "mock")]
    use crate::{prelude::*, result::ResultComp};

    #[test]
    #[cfg(feature = "mock")]
    fn begin() -> ResultComp<()> {
        use crate::{Syncpoint, result::Completion, test::mock, types::MQBO};

        let mock_connection = mock::connect_ok(|mock_library| {
            mock_library
                .expect_MQBEGIN()
                .returning(|_, _, cc, rc| mock::mqi_outcome_ok(cc, rc))
                .once();
            mock_library
                .expect_MQCMIT()
                .returning(|_, cc, rc| mock::mqi_outcome_ok(cc, rc))
                .once();
        });

        let sync = Syncpoint::begin(mock_connection, MQBO::default()).warn_as_error()?;
        sync.commit().warn_as_error()?;

        Ok(Completion::new(()))
    }
}
