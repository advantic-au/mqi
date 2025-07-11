use super::{
    AsyncPutStat, ReconnectionErrorStat, ReconnectionStat, stat_put, stat_reconnection,
    stat_reconnection_error,
};
use crate::{ResultComp, types::MQPMO, put_message_with};
use crate::{open, connection, put};

/// A trait that provides functions to put messages to a queue manager and inquire on the status of a previous MQI call or connection
pub trait QueueManager {
    /// Put a message to a queue or topic with a specified return type that implements [`PutAttr`](option::PutAttr).
    ///
    /// Type inference of the return value may not always work so you may have to explicitly state the return type using the
    /// `put_message_with::<Type>` syntax.
    fn put_message_with<'po, 'oo, R>(
        &self,
        open_options: &impl open::OpenOption<'oo, MQPMO>,
        put_options: &impl put::PutOption<'po>,
        message: &(impl put::PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: put::PutAttr;

    /// Put a message to a queue or topic
    #[inline]
    fn put_message<'po, 'oo>(
        &self,
        open_options: &impl open::OpenOption<'oo, MQPMO>,
        put_options: &impl put::PutOption<'po>,
        message: &(impl put::PutMessage + ?Sized),
    ) -> ResultComp<()> {
        self.put_message_with(open_options, put_options, message)
    }

    fn stat_put(&self) -> ResultComp<AsyncPutStat>;
    fn stat_reconnection(&self) -> ResultComp<ReconnectionStat>;
    fn stat_reconnection_error(&self) -> ResultComp<ReconnectionErrorStat>;
}

impl<C: connection::Conn> QueueManager for C {
    #[inline]
    fn put_message_with<'po, 'oo, R>(
        &self,
        open_options: &impl open::OpenOption<'oo, MQPMO>,
        put_options: &impl put::PutOption<'po>,
        message: &(impl put::PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: put::PutAttr,
    {
        put_message_with(self.mq(), self.handle(), open_options, put_options, message)
    }

    #[inline]
    fn stat_put(&self) -> ResultComp<AsyncPutStat> {
        stat_put(self.mq(), self.handle())
    }

    #[inline]
    fn stat_reconnection(&self) -> ResultComp<ReconnectionStat> {
        stat_reconnection(self.mq(), self.handle())
    }

    #[inline]
    fn stat_reconnection_error(&self) -> ResultComp<ReconnectionErrorStat> {
        stat_reconnection_error(self.mq(), self.handle())
    }
}
