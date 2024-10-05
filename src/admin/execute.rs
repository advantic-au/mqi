use libmqm_sys::{function, Mqai};

use crate::core::mqai::BagHandle;
use crate::core::ObjectHandle;
use crate::macros::all_option_tuples;
use crate::{prelude::*, Object};
use crate::{
    core::Library,
    sys,
    values::{MQCBO, MQCMD},
    Conn, ResultComp,
};

use super::{Bag, BagDrop, Owned};

#[derive(Debug, Default)]
pub struct ExecuteParam<'a> {
    command: MQCMD,
    options: Option<&'a BagHandle>,
    admin_object: Option<&'a ObjectHandle>,
    reply_object: Option<&'a ObjectHandle>,
}

#[derive(Debug, Clone, Copy)]
pub struct OptionsBag<'a, B: BagDrop, L: Library<MQ: Mqai>>(&'a Bag<B, L>);
#[derive(Debug, Clone, Copy)]
pub struct ReplyObject<'a, C: Conn>(&'a Object<C>);
#[derive(Debug, Clone, Copy)]
pub struct AdminObject<'a, C: Conn>(&'a Object<C>);

all_option_tuples!('e, ExecuteOption, ExecuteParam<'e>);

impl<'a, B: BagDrop, L: Library<MQ: Mqai>> ExecuteOption<'a> for OptionsBag<'a, B, L> {
    fn apply_param(self, param: &mut ExecuteParam<'a>) {
        param.options.replace(self.0.handle());
    }
}

impl<'a, C: Conn> ExecuteOption<'a> for ReplyObject<'a, C> {
    fn apply_param(self, param: &mut ExecuteParam<'a>) {
        param.reply_object.replace(self.0.handle());
    }
}

impl<'a, C: Conn> ExecuteOption<'a> for AdminObject<'a, C> {
    fn apply_param(self, param: &mut ExecuteParam<'a>) {
        param.reply_object.replace(self.0.handle());
    }
}

impl ExecuteOption<'_> for MQCMD {
    fn apply_param(self, param: &mut ExecuteParam) {
        param.command = self;
    }
}

pub trait ExecuteOption<'a> {
    fn apply_param(self, param: &mut ExecuteParam<'a>);
}

pub trait QueueManagerAdmin: Conn<Lib: Library<MQ: Mqai>> {
    fn execute<'a>(
        &self,
        admin: &Bag<impl BagDrop, Self::Lib>,
        options: impl ExecuteOption<'a>,
    ) -> ResultComp<Bag<Owned, Self::Lib>>;
}

impl<C> QueueManagerAdmin for C
where
    C: Conn<Lib: Library<MQ: function::Mqai> + Clone>, // A clonable connnection that supports MQAI functions
{
    fn execute<'a>(
        &self,
        admin: &Bag<impl BagDrop, Self::Lib>,
        options: impl ExecuteOption<'a>,
    ) -> ResultComp<Bag<Owned, Self::Lib>> {
        let lib = self.mq().0.clone();
        // There shouldn't be any warnings for creating a bag - so treat the warning as an error
        let response_bag = Bag::new_lib(lib, MQCBO(sys::MQCBO_ADMIN_BAG)).warn_as_error()?;

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
