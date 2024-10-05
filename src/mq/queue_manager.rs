use crate::{
    put::{PutAttr, PutMessage, PutOption},
    ResultComp,
};

use super::{
    put::put_message_with, stat_put, stat_reconnection, stat_reconnection_error, values::MQPMO, AsyncPutStat, Conn, OpenOption,
    ReconnectionErrorStat, ReconnectionStat,
};

pub trait QueueManager {
    fn put_message_with<'oo, R>(
        &self,
        open_options: impl OpenOption<'oo, MQPMO>,
        put_options: impl PutOption,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: PutAttr;

    #[inline]
    fn put_message<'oo>(
        &self,
        open_options: impl OpenOption<'oo, MQPMO>,
        put_options: impl PutOption,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<()> {
        self.put_message_with(open_options, put_options, message)
    }

    fn stat_put(&self) -> ResultComp<AsyncPutStat>;
    fn stat_reconnection(&self) -> ResultComp<ReconnectionStat>;
    fn stat_reconnection_error(&self) -> ResultComp<ReconnectionErrorStat>;
}

impl<C: Conn> QueueManager for C {
    #[inline]
    fn put_message_with<'oo, R>(
        &self,
        open_options: impl OpenOption<'oo, MQPMO>,
        put_options: impl PutOption,
        message: &(impl PutMessage + ?Sized),
    ) -> ResultComp<R>
    where
        R: PutAttr,
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
