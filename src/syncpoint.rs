use libmqm_default as default;

use crate::{
    connection::AsConnection,
    result::{ResultComp, ResultCompErrExt},
    structs,
    types::MQBO,
};

#[must_use]
#[derive(Debug)]
pub struct Syncpoint<C: AsConnection> {
    connection: C,
}

impl<C: AsConnection> Syncpoint<C> {
    pub const fn new(connection: C) -> Self {
        Self { connection }
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
    pub fn commit(self) -> ResultComp<()> {
        let mut mut_self = self;
        let conn = mut_self.connection.as_connection();
        let result = conn.mq.mqcmit(conn.handle);

        unsafe {
            std::ptr::drop_in_place(&raw mut mut_self.connection);
        }

        let _ = std::mem::ManuallyDrop::new(mut_self);
        result
    }

    /// This function uses the [`MQBACK`](libmqm_sys::MQBACK) verb.
    pub fn backout(self) -> ResultComp<()> {
        let mut mut_self = self;
        let conn = mut_self.connection.as_connection();
        let result = conn.mq.mqback(conn.handle);

        unsafe {
            std::ptr::drop_in_place(&raw mut mut_self.connection);
        }

        let _ = std::mem::ManuallyDrop::new(mut_self);
        result
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
