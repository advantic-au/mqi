use libmqm_default as default;

use crate::{Conn, result::ResultComp, result::ResultCompErrExt, structs, types::MQBO};

#[derive(Debug, PartialEq)]
enum SyncpointState {
    Open,
    Committed,
    Backout,
}

#[must_use]
pub struct Syncpoint<C: Conn> {
    state: SyncpointState,
    connection: C,
}

impl<C: Conn> Syncpoint<C> {
    pub const fn new(connection: C) -> Self {
        Self {
            state: SyncpointState::Open,
            connection,
        }
    }

    /// Begins a unit of work that is coordinated by the queue manager, and that can involve external resource managers.
    ///
    /// This function uses the [`MQBEGIN`](libmqm_sys::MQBEGIN) MQ API function.
    pub fn begin(connection: C, mqbo: MQBO) -> ResultComp<Self> {
        let mut bo = structs::MQBO::new(libmqm_sys::MQBO {
            Options: mqbo.0,
            ..default::MQBO_DEFAULT
        });
        connection
            .mq()
            .mqbegin(connection.handle(), Some(&mut bo))
            .map_completion(|()| Self::new(connection))
    }

    /// This function uses the [`MQCMIT`](libmqm_sys::MQCMIT) MQ API function.
    pub fn commit(self) -> ResultComp<()> {
        let result = self.connection.mq().mqcmit(self.connection.handle());
        let mut self_mut = self;
        self_mut.state = SyncpointState::Committed;
        result
    }

    /// This function uses the [`MQBACK`](libmqm_sys::MQBACK) MQ API function.
    pub fn backout(self) -> ResultComp<()> {
        let result = self.connection.mq().mqback(self.connection.handle());
        let mut self_mut = self;
        self_mut.state = SyncpointState::Backout;
        result
    }
}

impl<C: Conn> Drop for Syncpoint<C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if self.state == SyncpointState::Open {
            let _ = self.connection.mq().mqback(self.connection.handle());
        }
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    #[cfg(feature = "mock")]
    use crate::{result::ResultComp, prelude::*};

    #[test]
    #[cfg(feature = "mock")]
    fn begin() -> ResultComp<()> {
        use crate::{result::Completion, Syncpoint, test::mock, types::MQBO};

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
