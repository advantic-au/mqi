use libmqm_sys::{Mqai, Mqi};

use super::option::{ExecuteOption, ExecuteParam};
use crate::{
    Connection, Library,
    bag::{Bag, BagDrop, Owned},
    constants,
    prelude::*,
    result::ResultComp,
};

impl<L: Library<MQ: Mqi + Mqai> + Clone, H> Connection<L, H> {
    pub fn execute<'a>(&self, admin: &Bag<impl BagDrop, L>, options: &impl ExecuteOption<'a>) -> ResultComp<Bag<Owned, L>> {
        let functions = self.mq();
        let handle = self.handle();
        // There shouldn't be any warnings for creating a bag - so treat the warning as an error
        let response_bag = Bag::new_lib(functions.0.clone(), constants::MQCBO_ADMIN_BAG).warn_as_error()?;

        let mut param = ExecuteParam::default();
        options.apply_param(&mut param);

        functions
            .mq_execute(
                handle,
                param.command,
                param.options,
                admin,
                response_bag.handle(),
                param.admin_object,
                param.reply_object,
            )
            .map_completion(|()| response_bag)
    }
}
