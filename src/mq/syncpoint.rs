use crate::ResultComp;

use super::Connection;

#[derive(Debug, PartialEq)]
enum SyncpointState {
    Open,
    Committed,
    Backout,
}

#[must_use]
pub struct Syncpoint<C: Connection> {
    state: SyncpointState,
    connection: C,
}

impl<C: Connection> Syncpoint<C> {
    pub const fn new(connection: C) -> Self {
        Self {
            state: SyncpointState::Open,
            connection,
        }
    }

    pub fn commit(self) -> ResultComp<()> {
        let result = self.connection.mq().mqcmit(self.connection.handle());
        let mut self_mut = self;
        self_mut.state = SyncpointState::Committed;
        result
    }

    pub fn backout(self) -> ResultComp<()> {
        let result = self.connection.mq().mqback(self.connection.handle());
        let mut self_mut = self;
        self_mut.state = SyncpointState::Backout;
        result
    }
}

impl<C: Connection> Drop for Syncpoint<C> {
    fn drop(&mut self) {
        // TODO: handle close failure
        if self.state == SyncpointState::Open {
            let _ = self.connection.mq().mqback(self.connection.handle());
        }
    }
}
