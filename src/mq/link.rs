use libmqm_sys::link::LinkedMQ;

use super::{ConnectionOptions, ConnectionShare, HandleShare};
use crate::{sys, DefinitionMethod, QMName, ResultComp, StructBuilder, StructOptionBuilder};

#[cfg(feature = "mqai")]
use crate::{
    admin::{Bag, Owned},
    core::mqai::CreateBagOptions,
    Mask, ResultErr
};

impl<C: StructOptionBuilder<sys::MQCSP>, D: DefinitionMethod> ConnectionOptions<C, D> {
    pub fn connect<H: HandleShare>(self, qm_name: Option<&QMName>) -> ResultComp<ConnectionShare<&LinkedMQ, H>> {
        self.connect_lib(&LinkedMQ, qm_name)
    }
}

impl<H: HandleShare> ConnectionShare<&LinkedMQ, H> {
    pub fn new(
        qm_name: Option<&QMName>,
        builder: &impl StructBuilder<sys::MQCNO>,
    ) -> ResultComp<Self> {
        Self::new_lib(&LinkedMQ, qm_name, builder)
    }
}

#[cfg(feature = "mqai")]
impl Bag<Owned, &LinkedMQ> {
    pub fn new(options: Mask<CreateBagOptions>) -> ResultErr<Self> {
        Self::new_lib(&LinkedMQ, options)
    }
}
