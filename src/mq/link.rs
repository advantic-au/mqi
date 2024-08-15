use libmqm_sys::link::LinkedMQ;

use super::{connect_options::ConnectOptions, types::QueueManagerName, ConnectParam, HandleShare, QueueManagerShare, MqiValue};
use crate::{core::values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::{
    admin::{Bag, Owned},
    MqMask,
};

impl<H: HandleShare> QueueManagerShare<'_, LinkedMQ, H> {
    #[allow(clippy::new_ret_no_self)]
    pub fn connect<'c, R>(qm_name: Option<&QueueManagerName>, options: &impl ConnectOptions<'c>) -> ResultComp<R>
    where
        R: for<'a> MqiValue<Self, Param<'a> = ConnectParam<'a>>,
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
