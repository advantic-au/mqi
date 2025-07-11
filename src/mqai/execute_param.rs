use libmqm_constants::types::MQCMD;
use libmqm_sys::Mqai;

use crate::{Library, Object, mqai::option, connection::Conn};

use super::{Bag, BagDrop};

#[derive(Debug, Clone, Copy)]
pub struct OptionsBag<'a, B: BagDrop, L: Library<MQ: Mqai>>(&'a Bag<B, L>);
#[derive(Debug, Clone, Copy)]
pub struct ReplyObject<'a, C: Conn>(&'a Object<C>);
#[derive(Debug, Clone, Copy)]
pub struct AdminObject<'a, C: Conn>(&'a Object<C>);

impl<'a, B: super::BagDrop, L: Library<MQ: Mqai>> option::ExecuteOption<'a> for OptionsBag<'a, B, L> {
    fn apply_param(&self, param: &mut option::ExecuteParam<'a>) {
        param.options.replace(self.0.handle());
    }
}

impl<'a, C: Conn> option::ExecuteOption<'a> for ReplyObject<'a, C> {
    fn apply_param(&self, param: &mut option::ExecuteParam<'a>) {
        param.reply_object.replace(self.0.handle());
    }
}

impl<'a, C: Conn> option::ExecuteOption<'a> for AdminObject<'a, C> {
    fn apply_param(&self, param: &mut option::ExecuteParam<'a>) {
        param.reply_object.replace(self.0.handle());
    }
}

impl option::ExecuteOption<'_> for MQCMD {
    fn apply_param(&self, param: &mut option::ExecuteParam) {
        param.command = *self;
    }
}
