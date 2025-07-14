use libmqm_sys::Mqai;

use super::option::{ExecuteOption, ExecuteParam};
use crate::{
    Library, MqFunctions, result::ResultComp,
    bag::{Bag, BagDrop, Owned},
    constants,
    handle::ConnectionHandle,
    prelude::*,
};

pub fn execute<'a, L: Library<MQ: Mqai> + Clone>(
    functions: &MqFunctions<L>,
    handle: ConnectionHandle,
    admin: &Bag<impl BagDrop, L>,
    options: &impl ExecuteOption<'a>,
) -> ResultComp<Bag<Owned, L>> {
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
