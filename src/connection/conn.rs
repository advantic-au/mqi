use libmqm_sys::Mqi;

use crate::callback::function::register_event_handler;
use crate::put::function::put_message_with;
use crate::stat::function::{stat_put, stat_reconnection, stat_reconnection_error};
use crate::stat::{AsyncPutStat, ReconnectionErrorStat, ReconnectionStat};
use crate::{ConnectionHandle, ConnectionRef, Error, Library, MqFunctions, open, put, structs, types};
use crate::{ResultComp, types::MQPMO};

/// Associated connection handle and MQ library
pub trait Conn {
    type Lib: Library<MQ: Mqi>;
    type Thread;

    fn mq(&self) -> &MqFunctions<Self::Lib>;
    fn handle(&self) -> ConnectionHandle;

    /// # Safety
    /// Consumers of [`register_event_handler`](Self::register_event_handler) must handle and read the pointers in [`MQCBC`](structs::MQCBC) correctly
    unsafe fn register_event_handler<F>(&self, options: types::MQCBDO, closure: F) -> Result<(), Error>
    where
        Self::Lib: Clone,
        F: FnMut(ConnectionRef<Self::Lib, Self::Thread>, &structs::MQCBC),
    {
        unsafe { register_event_handler(self.mq(), self.handle(), options, closure) }
    }

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

    /// Put a message to a queue or topic with a specified return type that implements [`PutAttr`](put::PutAttr).
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
