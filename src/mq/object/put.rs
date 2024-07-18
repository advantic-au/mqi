use libmqm_sys::function;

use crate::{sys, Conn, Object, QueueManagerShare, ResultComp};
use crate::core;

use crate::MQMD;

impl<C: Conn> Object<C> {
    // TODO: deal with optional mqmd
    pub fn put<B>(&self, mqmd: &mut impl MQMD, pmo: &mut sys::MQPMO, body: &B) -> ResultComp<()> {
        self.connection()
            .mq()
            .mqput(self.connection().handle(), self.handle(), Some(mqmd), pmo, body)
    }
}

impl<L: core::Library<MQ: function::MQI>, H> QueueManagerShare<'_, L, H> {
    pub fn put<B>(&self, mqod: &mut sys::MQOD, mqmd: Option<&mut impl MQMD>, pmo: &mut sys::MQPMO, body: &B) -> ResultComp<()> {
        self.mq().mqput1(self.handle(), mqod, mqmd, pmo, body)
    }
}
