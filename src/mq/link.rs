use libmqm_sys::link::LinkedMQ;

use super::{connect_options::ConnectOption, types::QueueManagerName, HandleShare, QueueManagerShare, ConnectValue};
use crate::{core::values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::{
    admin::{Bag, Owned},
};

impl<H: HandleShare> QueueManagerShare<'_, LinkedMQ, H> {
    pub fn connect<'co, R>(qm_name: Option<&QueueManagerName>, options: &impl ConnectOption<'co>) -> ResultComp<R>
    where
        R: ConnectValue<Self>,
    {
        Self::connect_lib(LinkedMQ, qm_name, options)
    }
}

#[cfg(feature = "mqai")]
impl Bag<Owned, LinkedMQ> {
    pub fn new(options: MQCBO) -> ResultComp<Self> {
        Self::connect_lib(LinkedMQ, options)
    }
}
