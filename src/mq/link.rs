use libmqm_sys::link::LinkedMQ;

use super::{connect_options::ConnectOption, types::QueueManagerName, HandleShare, QueueManagerShare, QueueManagerValue};
use crate::{core::values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::{
    admin::{Bag, Owned},
    MqMask,
};

impl<H: HandleShare> QueueManagerShare<'_, LinkedMQ, H> {
    #[allow(clippy::new_ret_no_self)]
    pub fn connect<'co, R>(qm_name: Option<&QueueManagerName>, options: &impl ConnectOption<'co>) -> ResultComp<R>
    where
        R: QueueManagerValue<Self>,
    {
        Self::connect_lib(LinkedMQ, qm_name, options)
    }
}

#[cfg(feature = "mqai")]
impl Bag<Owned, LinkedMQ> {
    pub fn new(options: MqMask<MQCBO>) -> ResultComp<Self> {
        Self::connect_lib(LinkedMQ, options)
    }
}
