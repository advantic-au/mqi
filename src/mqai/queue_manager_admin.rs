use libmqm_constants::constants;
use libmqm_sys::Mqai;

use crate::mqai::{Bag, BagDrop, Owned};
use crate::{
    Library, ResultComp,
    mqai::option::{ExecuteOption, ExecuteParam},
    connection::Conn,
    prelude::*,
};

pub trait QueueManagerAdmin: Conn<Lib: Library<MQ: Mqai>> {
    /// This function uses the [`mqExecute`](libmqm_sys::mqai::mqExecute) MQ API function
    fn execute<'a>(
        &self,
        admin: &Bag<impl BagDrop, Self::Lib>,
        options: &impl ExecuteOption<'a>,
    ) -> ResultComp<Bag<Owned, Self::Lib>>;
}

impl<C> QueueManagerAdmin for C
where
    C: Conn<Lib: Library<MQ: Mqai> + Clone>, // A clonable connnection that supports MQAI functions
{
    fn execute<'a>(
        &self,
        admin: &Bag<impl BagDrop, Self::Lib>,
        options: &impl ExecuteOption<'a>,
    ) -> ResultComp<Bag<Owned, Self::Lib>> {
        let lib = self.mq().0.clone();
        // There shouldn't be any warnings for creating a bag - so treat the warning as an error
        let response_bag = Bag::new_lib(lib, constants::MQCBO_ADMIN_BAG).warn_as_error()?;

        let mut param = ExecuteParam::default();
        options.apply_param(&mut param);

        self.mq()
            .mq_execute(
                self.handle(),
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
