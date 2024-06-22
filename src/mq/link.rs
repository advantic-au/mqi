use libmqm_sys::link::LinkedMQ;

use super::{ConnectionOptions, QueueManagerShare, HandleShare};
use crate::{sys, ConnectionId, DefinitionMethod, QMName, ResultComp, StructBuilder, StructOptionBuilder};

#[cfg(feature = "mqai")]
use crate::{
    admin::{Bag, Owned},
    core::mqai::MQCBO,
    MqMask,
};

impl<C: StructOptionBuilder<sys::MQCSP>, D: DefinitionMethod> ConnectionOptions<C, D> {
    pub fn connect<H: HandleShare>(self, qm_name: Option<&QMName>) -> ResultComp<(QueueManagerShare<&LinkedMQ, H>, ConnectionId, Option<String>)> {
        self.connect_lib(&LinkedMQ, qm_name)
    }
}

impl<H: HandleShare> QueueManagerShare<&LinkedMQ, H> {
    pub fn new(
        qm_name: Option<&QMName>,
        builder: &impl StructBuilder<sys::MQCNO>,
    ) -> ResultComp<(Self, ConnectionId, Option<String>)> {
        Self::new_lib(&LinkedMQ, qm_name, builder)
    }
}

#[cfg(feature = "mqai")]
impl Bag<Owned, &LinkedMQ> {
    pub fn new(options: MqMask<MQCBO>) -> ResultComp<Self> {
        Self::new_lib(&LinkedMQ, options)
    }
}
