use libmqm_sys::link::LinkedMQ;

use super::{connect_options::CnoOptions, types::QueueName, ConnectValue, HandleShare, QueueManagerShare};
use crate::{core::values::MQCBO, ResultComp};

#[cfg(feature = "mqai")]
use crate::{
    admin::{Bag, Owned},
    MqMask,
};

// impl<C: StructOptionBuilder<sys::MQCSP>, D: DefinitionMethod> ConnectionOptions<C, D> {
//     pub fn connect<'cb, R: ConnectValue<QueueManagerShare<'cb, &'static LinkedMQ, H>>, H: HandleShare>(
//         self,
//         qm_name: Option<&QMName>,
//     ) -> ResultComp<R> {
//         self.connect_lib(&LinkedMQ, qm_name)
//     }
// }

impl<H: HandleShare> QueueManagerShare<'_, &LinkedMQ, H> {
    #[allow(clippy::new_ret_no_self)]
    pub fn connect<'c, R: ConnectValue<Self>> (
        qm_name: Option<&QueueName>,
        options: impl CnoOptions<'c>,
    ) -> ResultComp<R> {
        Self::new_lib(&LinkedMQ, qm_name, options)
    }
}

#[cfg(feature = "mqai")]
impl Bag<Owned, &LinkedMQ> {
    pub fn new(options: MqMask<MQCBO>) -> ResultComp<Self> {
        Self::new_lib(&LinkedMQ, options)
    }
}
